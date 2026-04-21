use crate::ast::{Expr, Stmt};
use crate::diagnostics::{DiagnosticError, ErrorPhase, Result};
use crate::token::{Token, TokenType};
use crate::value::Value;

fn escape_c_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn sanitize_ident(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn unsupported(token: Option<&Token>, message: impl Into<String>) -> DiagnosticError {
    if let Some(token) = token {
        DiagnosticError::new(ErrorPhase::Runtime, token.line, token.column, message)
    } else {
        DiagnosticError::new(ErrorPhase::Runtime, 0, 0, message)
    }
}

fn emit_expr(expression: &Expr) -> Result<String> {
    match expression {
        Expr::Assign { name, value } => Ok(format!(
            "({} = {})",
            sanitize_ident(&name.lexeme),
            emit_expr(value)?
        )),
        Expr::Binary { left, op, right } => {
            let left = emit_expr(left)?;
            let right = emit_expr(right)?;
            let call = match op.kind {
                TokenType::Plus => format!("idot_add({left}, {right})"),
                TokenType::Minus => format!("idot_sub({left}, {right})"),
                TokenType::Star => format!("idot_mul({left}, {right})"),
                TokenType::Slash => format!("idot_div({left}, {right})"),
                TokenType::Percent => format!("idot_mod({left}, {right})"),
                TokenType::EqualEqual => format!("idot_eq({left}, {right})"),
                TokenType::BangEqual => format!("idot_neq({left}, {right})"),
                TokenType::Less => format!("idot_lt({left}, {right})"),
                TokenType::LessEqual => format!("idot_lte({left}, {right})"),
                TokenType::Greater => format!("idot_gt({left}, {right})"),
                TokenType::GreaterEqual => format!("idot_gte({left}, {right})"),
                _ => {
                    return Err(unsupported(
                        Some(op),
                        "Unsupported binary operator in emitter.",
                    ))
                }
            };
            Ok(call)
        }
        Expr::Grouping(inner) => emit_expr(inner),
        Expr::Literal(value) => Ok(match value {
            Value::Nil => "idot_nil()".to_string(),
            Value::Number(number) => format!("idot_num({number:.15})"),
            Value::Bool(boolean) => {
                if *boolean {
                    "idot_bool(1)".to_string()
                } else {
                    "idot_bool(0)".to_string()
                }
            }
            Value::String(text) => format!("idot_str(\"{}\")", escape_c_string(text)),
        }),
        Expr::Unary { op, right } => {
            let right = emit_expr(right)?;
            let call = match op.kind {
                TokenType::Bang => format!("idot_bool(!idot_is_truthy({right}))"),
                TokenType::Minus => format!("idot_num(-idot_require_num({right}))"),
                _ => {
                    return Err(unsupported(
                        Some(op),
                        "Unsupported unary operator in emitter.",
                    ))
                }
            };
            Ok(call)
        }
        Expr::Variable(name) => Ok(sanitize_ident(&name.lexeme)),
    }
}

fn emit_stmt(statement: &Stmt, indent: usize, out: &mut String) -> Result<()> {
    let pad = "    ".repeat(indent);
    match statement {
        Stmt::Block(statements) => {
            out.push_str(&format!("{pad}{{\n"));
            for statement in statements {
                emit_stmt(statement, indent + 1, out)?;
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        Stmt::Expr(expression) => {
            out.push_str(&format!("{pad}(void)({});\n", emit_expr(expression)?));
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            out.push_str(&format!(
                "{pad}if (idot_is_truthy({})) ",
                emit_expr(condition)?
            ));
            match &**then_branch {
                Stmt::Block(statements) => {
                    out.push_str("{\n");
                    for statement in statements {
                        emit_stmt(statement, indent + 1, out)?;
                    }
                    out.push_str(&format!("{pad}}}"));
                }
                _ => {
                    out.push_str("{\n");
                    emit_stmt(then_branch, indent + 1, out)?;
                    out.push_str(&format!("{pad}}}"));
                }
            }

            if let Some(else_branch) = else_branch {
                out.push_str(" else ");
                match &**else_branch {
                    Stmt::Block(statements) => {
                        out.push_str("{\n");
                        for statement in statements {
                            emit_stmt(statement, indent + 1, out)?;
                        }
                        out.push_str(&format!("{pad}}}\n"));
                    }
                    _ => {
                        out.push_str("{\n");
                        emit_stmt(else_branch, indent + 1, out)?;
                        out.push_str(&format!("{pad}}}\n"));
                    }
                }
            } else {
                out.push('\n');
            }
        }
        Stmt::Print(expression) => {
            out.push_str(&format!("{pad}idot_print({});\n", emit_expr(expression)?));
        }
        Stmt::Var { name, initializer } => {
            out.push_str(&format!(
                "{pad}IdotValue {} = {};\n",
                sanitize_ident(&name.lexeme),
                emit_expr(initializer)?
            ));
        }
    }
    Ok(())
}

pub fn emit_c(statements: &[Stmt]) -> Result<String> {
    let mut out = String::new();
    out.push_str(
        r#"#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef enum {
    IDOT_NIL,
    IDOT_NUM,
    IDOT_BOOL,
    IDOT_STR
} IdotTag;

typedef struct {
    IdotTag tag;
    double number;
    int boolean;
    char* string;
} IdotValue;

static void idot_runtime_error(const char* message) {
    fprintf(stderr, "Runtime error: %s\n", message);
    exit(1);
}

static char* idot_strdup(const char* source) {
    size_t length = strlen(source);
    char* output = (char*)malloc(length + 1);
    if (!output) {
        idot_runtime_error("Out of memory");
    }
    memcpy(output, source, length + 1);
    return output;
}

static IdotValue idot_nil(void) {
    IdotValue value;
    value.tag = IDOT_NIL;
    value.number = 0;
    value.boolean = 0;
    value.string = NULL;
    return value;
}

static IdotValue idot_num(double number) {
    IdotValue value;
    value.tag = IDOT_NUM;
    value.number = number;
    value.boolean = 0;
    value.string = NULL;
    return value;
}

static IdotValue idot_bool(int boolean) {
    IdotValue value;
    value.tag = IDOT_BOOL;
    value.number = 0;
    value.boolean = boolean != 0;
    value.string = NULL;
    return value;
}

static IdotValue idot_str(const char* text) {
    IdotValue value;
    value.tag = IDOT_STR;
    value.number = 0;
    value.boolean = 0;
    value.string = idot_strdup(text);
    return value;
}

static int idot_is_truthy(IdotValue value) {
    switch (value.tag) {
        case IDOT_NIL:
            return 0;
        case IDOT_BOOL:
            return value.boolean;
        default:
            return 1;
    }
}

static double idot_require_num(IdotValue value) {
    if (value.tag != IDOT_NUM) {
        idot_runtime_error("Expected number.");
    }
    return value.number;
}

static IdotValue idot_add(IdotValue left, IdotValue right) {
    if (left.tag == IDOT_NUM && right.tag == IDOT_NUM) {
        return idot_num(left.number + right.number);
    }
    if (left.tag == IDOT_STR && right.tag == IDOT_STR) {
        size_t left_length = strlen(left.string);
        size_t right_length = strlen(right.string);
        char* joined = (char*)malloc(left_length + right_length + 1);
        if (!joined) {
            idot_runtime_error("Out of memory");
        }
        memcpy(joined, left.string, left_length);
        memcpy(joined + left_length, right.string, right_length + 1);
        IdotValue value;
        value.tag = IDOT_STR;
        value.number = 0;
        value.boolean = 0;
        value.string = joined;
        return value;
    }
    idot_runtime_error("Operator '+' requires two numbers or two strings.");
    return idot_nil();
}

static IdotValue idot_sub(IdotValue left, IdotValue right) {
    return idot_num(idot_require_num(left) - idot_require_num(right));
}

static IdotValue idot_mul(IdotValue left, IdotValue right) {
    return idot_num(idot_require_num(left) * idot_require_num(right));
}

static IdotValue idot_div(IdotValue left, IdotValue right) {
    double divisor = idot_require_num(right);
    if (divisor == 0.0) {
        idot_runtime_error("Division by zero.");
    }
    return idot_num(idot_require_num(left) / divisor);
}

static IdotValue idot_mod(IdotValue left, IdotValue right) {
    double divisor = idot_require_num(right);
    if (divisor == 0.0) {
        idot_runtime_error("Modulo by zero.");
    }
    return idot_num(fmod(idot_require_num(left), divisor));
}

static int idot_values_equal(IdotValue left, IdotValue right) {
    if (left.tag != right.tag) {
        return 0;
    }
    switch (left.tag) {
        case IDOT_NIL:
            return 1;
        case IDOT_NUM:
            return left.number == right.number;
        case IDOT_BOOL:
            return left.boolean == right.boolean;
        case IDOT_STR:
            return strcmp(left.string, right.string) == 0;
    }
    return 0;
}

static IdotValue idot_eq(IdotValue left, IdotValue right) {
    return idot_bool(idot_values_equal(left, right));
}

static IdotValue idot_neq(IdotValue left, IdotValue right) {
    return idot_bool(!idot_values_equal(left, right));
}

static IdotValue idot_lt(IdotValue left, IdotValue right) {
    return idot_bool(idot_require_num(left) < idot_require_num(right));
}

static IdotValue idot_lte(IdotValue left, IdotValue right) {
    return idot_bool(idot_require_num(left) <= idot_require_num(right));
}

static IdotValue idot_gt(IdotValue left, IdotValue right) {
    return idot_bool(idot_require_num(left) > idot_require_num(right));
}

static IdotValue idot_gte(IdotValue left, IdotValue right) {
    return idot_bool(idot_require_num(left) >= idot_require_num(right));
}

static void idot_print(IdotValue value) {
    switch (value.tag) {
        case IDOT_NIL:
            puts("nil");
            break;
        case IDOT_NUM:
            printf("%.15g\n", value.number);
            break;
        case IDOT_BOOL:
            puts(value.boolean ? "true" : "false");
            break;
        case IDOT_STR:
            puts(value.string);
            break;
    }
}

int main(void) {
"#,
    );

    for statement in statements {
        emit_stmt(statement, 1, &mut out)?;
    }

    out.push_str("    return 0;\n}\n");
    Ok(out)
}

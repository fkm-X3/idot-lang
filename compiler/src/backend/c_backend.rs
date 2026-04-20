fn escape_c_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

pub fn emit_c(stmts: &Vec<crate::lexer::Stmt>) -> Result<String, Box<dyn std::error::Error>> {
    use std::collections::HashSet;
    let mut out = String::new();
    out.push_str("#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n\ntypedef struct IdotValue { int is_str; long long num; char* str; } IdotValue;\n\nIdotValue idot_new_num(long long n) { IdotValue v; v.is_str = 0; v.num = n; v.str = NULL; return v; }\nIdotValue idot_new_str(const char* s) { IdotValue v; v.is_str = 1; v.str = strdup(s); v.num = 0; return v; }\nlong long idot_to_num(IdotValue v) { return v.num; }\nvoid idot_print(IdotValue v) { if (v.is_str) puts(v.str); else printf("%lld\\n", (long long)v.num); }\n\nint main(void) {\n");

    let mut declared: HashSet<String> = HashSet::new();

    fn sanitize_ident(s: &str) -> String {
        s.chars().map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' }).collect()
    }

    fn gen_expr_code(e: &crate::lexer::Expr, esc: &dyn Fn(&str) -> String) -> String {
        match e {
            crate::lexer::Expr::Number(n) => format!("idot_new_num({})", n),
            crate::lexer::Expr::Str(s) => format!("idot_new_str(\"{}\")", esc(s)),
            crate::lexer::Expr::Var(name) => sanitize_ident(name),
            crate::lexer::Expr::Unary(op, inner) => match op {
                crate::lexer::UnOp::Neg => format!("idot_new_num(- idot_to_num({}))", gen_expr_code(inner, esc)),
            },
            crate::lexer::Expr::Binary(left, op, right) => {
                let l = gen_expr_code(left, esc);
                let r = gen_expr_code(right, esc);
                let op_str = match op {
                    crate::lexer::BinOp::Add => "+",
                    crate::lexer::BinOp::Sub => "-",
                    crate::lexer::BinOp::Mul => "*",
                    crate::lexer::BinOp::Div => "/",
                    crate::lexer::BinOp::Eq => "==",
                    crate::lexer::BinOp::Neq => "!=",
                    crate::lexer::BinOp::Lt => "<",
                    crate::lexer::BinOp::LtEq => "<=",
                    crate::lexer::BinOp::Gt => ">",
                    crate::lexer::BinOp::GtEq => ">=",
                };
                format!("idot_new_num(idot_to_num({}) {} idot_to_num({}))", l, op_str, r)
            }
        }
    }

    for stmt in stmts {
        match stmt {
            crate::lexer::Stmt::Let(name, expr) => {
                let cname = sanitize_ident(name);
                declared.insert(cname.clone());
                let code = gen_expr_code(expr, &escape_c_string);
                out.push_str(&format!("    IdotValue {} = {};\n", cname, code));
            }
            crate::lexer::Stmt::Assign(name, expr) => {
                let cname = sanitize_ident(name);
                let code = gen_expr_code(expr, &escape_c_string);
                out.push_str(&format!("    {} = {};\n", cname, code));
            }
            crate::lexer::Stmt::Print(expr) => {
                let code = gen_expr_code(expr, &escape_c_string);
                out.push_str(&format!("    idot_print({});\n", code));
            }
            crate::lexer::Stmt::ExprStmt(expr) => {
                let code = gen_expr_code(expr, &escape_c_string);
                out.push_str(&format!("    (void)({});\n", code));
            }
        }
    }

    out.push_str("    return 0;\n}\n");
    Ok(out)
}

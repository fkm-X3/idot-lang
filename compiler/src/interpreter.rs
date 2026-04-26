use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use crate::ast::{Expr, Stmt};
use crate::diagnostics::{DiagnosticError, ErrorPhase, Result};
use crate::environment::Environment;
use crate::token::{Token, TokenType};
use crate::value::Value;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new(None))),
        }
    }

    pub fn execute(&mut self, statements: &[Stmt], output: &mut dyn Write) -> Result<()> {
        for statement in statements {
            self.execute_stmt(statement, output)?;
        }
        Ok(())
    }

    fn execute_stmt(&mut self, statement: &Stmt, output: &mut dyn Write) -> Result<()> {
        match statement {
            Stmt::Block(statements) => {
                let environment = Rc::new(RefCell::new(Environment::new(Some(
                    self.environment.clone(),
                ))));
                self.execute_block(statements, environment, output)
            }
            Stmt::Expr(expression) => {
                let _ = self.evaluate(expression)?;
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(condition)?.is_truthy() {
                    self.execute_stmt(then_branch, output)
                } else if let Some(branch) = else_branch {
                    self.execute_stmt(branch, output)
                } else {
                    Ok(())
                }
            }
            Stmt::Print(expression) => {
                let value = self.evaluate(expression)?;
                writeln!(output, "{}", value.to_idot_string())
                    .map_err(|error| Self::io_error("write failed", error))?;
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let value = self.evaluate(initializer)?;
                self.environment.borrow_mut().define(&name.lexeme, value);
                Ok(())
            }
        }
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Rc<RefCell<Environment>>,
        output: &mut dyn Write,
    ) -> Result<()> {
        let previous = self.environment.clone();
        self.environment = environment;

        let result = (|| {
            for statement in statements {
                self.execute_stmt(statement, output)?;
            }
            Ok(())
        })();

        self.environment = previous;
        result
    }

    fn evaluate(&mut self, expression: &Expr) -> Result<Value> {
        match expression {
            Expr::Assign { name, value } => {
                let assigned = self.evaluate(value)?;
                self.environment
                    .borrow_mut()
                    .assign(name, assigned.clone())?;
                Ok(assigned)
            }
            Expr::Binary { left, op, right } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
                match op.kind {
                    TokenType::Plus => match (left, right) {
                        (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs + rhs)),
                        (Value::String(lhs), Value::String(rhs)) => {
                            Ok(Value::String(format!("{lhs}{rhs}")))
                        }
                        _ => Err(Self::runtime_error(
                            op,
                            "Operator '+' requires two numbers or two strings.",
                        )),
                    },
                    TokenType::Minus => Ok(Value::Number(
                        self.require_number(left, op, "left operand")?
                            - self.require_number(right, op, "right operand")?,
                    )),
                    TokenType::Star => Ok(Value::Number(
                        self.require_number(left, op, "left operand")?
                            * self.require_number(right, op, "right operand")?,
                    )),
                    TokenType::Slash => {
                        let divisor = self.require_number(right, op, "right operand")?;
                        if divisor == 0.0 {
                            return Err(Self::runtime_error(op, "Division by zero."));
                        }
                        Ok(Value::Number(
                            self.require_number(left, op, "left operand")? / divisor,
                        ))
                    }
                    TokenType::Percent => {
                        let divisor = self.require_number(right, op, "right operand")?;
                        if divisor == 0.0 {
                            return Err(Self::runtime_error(op, "Modulo by zero."));
                        }
                        Ok(Value::Number(
                            self.require_number(left, op, "left operand")? % divisor,
                        ))
                    }
                    TokenType::Greater => Ok(Value::Bool(
                        self.require_number(left, op, "left operand")?
                            > self.require_number(right, op, "right operand")?,
                    )),
                    TokenType::GreaterEqual => Ok(Value::Bool(
                        self.require_number(left, op, "left operand")?
                            >= self.require_number(right, op, "right operand")?,
                    )),
                    TokenType::Less => Ok(Value::Bool(
                        self.require_number(left, op, "left operand")?
                            < self.require_number(right, op, "right operand")?,
                    )),
                    TokenType::LessEqual => Ok(Value::Bool(
                        self.require_number(left, op, "left operand")?
                            <= self.require_number(right, op, "right operand")?,
                    )),
                    TokenType::EqualEqual => Ok(Value::Bool(left == right)),
                    TokenType::BangEqual => Ok(Value::Bool(left != right)),
                    _ => Err(Self::runtime_error(op, "Unsupported binary operator.")),
                }
            }
            Expr::Call { callee, args } => {
                self.call_function(&callee.lexeme, args)
            }
            Expr::Grouping(inner) => self.evaluate(inner),
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Unary { op, right } => {
                let right = self.evaluate(right)?;
                match op.kind {
                    TokenType::Bang => Ok(Value::Bool(!right.is_truthy())),
                    TokenType::Minus => {
                        Ok(Value::Number(-self.require_number(right, op, "operand")?))
                    }
                    _ => Err(Self::runtime_error(op, "Unsupported unary operator.")),
                }
            }
            Expr::Variable(name) => self.environment.borrow().get(name),
        }
    }

    fn call_function(&mut self, name: &str, args: &[Expr]) -> Result<Value> {
        // Evaluate arguments
        let mut evaluated_args = Vec::new();
        for arg in args {
            evaluated_args.push(self.evaluate(arg)?);
        }

        // Call built-in graphics functions
        match name {
            "create_window" => {
                if evaluated_args.len() != 2 {
                    return Err(DiagnosticError::new(
                        ErrorPhase::Runtime,
                        0,
                        0,
                        format!("create_window expects 2 arguments, got {}", evaluated_args.len()),
                    ));
                }
                let width = self.require_number_from_value(&evaluated_args[0])?;
                let height = self.require_number_from_value(&evaluated_args[1])?;
                idot_graphics::create_window(width as u32, height as u32);
                Ok(Value::Nil)
            }
            "draw_rect" => {
                if evaluated_args.len() != 5 {
                    return Err(DiagnosticError::new(
                        ErrorPhase::Runtime,
                        0,
                        0,
                        format!("draw_rect expects 5 arguments, got {}", evaluated_args.len()),
                    ));
                }
                let x = self.require_number_from_value(&evaluated_args[0])?;
                let y = self.require_number_from_value(&evaluated_args[1])?;
                let width = self.require_number_from_value(&evaluated_args[2])?;
                let height = self.require_number_from_value(&evaluated_args[3])?;
                let color = self.require_string_from_value(&evaluated_args[4])?;
                idot_graphics::draw_rect(x as f32, y as f32, width as f32, height as f32, &color);
                Ok(Value::Nil)
            }
            "draw_circle" => {
                if evaluated_args.len() != 4 {
                    return Err(DiagnosticError::new(
                        ErrorPhase::Runtime,
                        0,
                        0,
                        format!("draw_circle expects 4 arguments, got {}", evaluated_args.len()),
                    ));
                }
                let x = self.require_number_from_value(&evaluated_args[0])?;
                let y = self.require_number_from_value(&evaluated_args[1])?;
                let radius = self.require_number_from_value(&evaluated_args[2])?;
                let color = self.require_string_from_value(&evaluated_args[3])?;
                idot_graphics::draw_circle(x as f32, y as f32, radius as f32, &color);
                Ok(Value::Nil)
            }
            "draw_line" => {
                if evaluated_args.len() != 5 {
                    return Err(DiagnosticError::new(
                        ErrorPhase::Runtime,
                        0,
                        0,
                        format!("draw_line expects 5 arguments, got {}", evaluated_args.len()),
                    ));
                }
                let x1 = self.require_number_from_value(&evaluated_args[0])?;
                let y1 = self.require_number_from_value(&evaluated_args[1])?;
                let x2 = self.require_number_from_value(&evaluated_args[2])?;
                let y2 = self.require_number_from_value(&evaluated_args[3])?;
                let color = self.require_string_from_value(&evaluated_args[4])?;
                idot_graphics::draw_line(x1 as f32, y1 as f32, x2 as f32, y2 as f32, &color);
                Ok(Value::Nil)
            }
            "clear_window" => {
                if evaluated_args.len() != 1 {
                    return Err(DiagnosticError::new(
                        ErrorPhase::Runtime,
                        0,
                        0,
                        format!("clear_window expects 1 argument, got {}", evaluated_args.len()),
                    ));
                }
                let color = self.require_string_from_value(&evaluated_args[0])?;
                idot_graphics::clear_window(&color);
                Ok(Value::Nil)
            }
            _ => Err(DiagnosticError::new(
                ErrorPhase::Runtime,
                0,
                0,
                format!("Unknown function: '{}'", name),
            )),
        }
    }

    fn require_number_from_value(&self, value: &Value) -> Result<f64> {
        if let Value::Number(number) = value {
            return Ok(*number);
        }
        Err(DiagnosticError::new(
            ErrorPhase::Runtime,
            0,
            0,
            "Expected number".to_string(),
        ))
    }

    fn require_string_from_value(&self, value: &Value) -> Result<String> {
        if let Value::String(s) = value {
            return Ok(s.clone());
        }
        Err(DiagnosticError::new(
            ErrorPhase::Runtime,
            0,
            0,
            "Expected string".to_string(),
        ))
    }

    fn require_number(&self, value: Value, token: &Token, context: &str) -> Result<f64> {
        if let Value::Number(number) = value {
            return Ok(number);
        }
        Err(Self::runtime_error(
            token,
            format!("Expected number for {context}."),
        ))
    }

    fn runtime_error(token: &Token, message: impl Into<String>) -> DiagnosticError {
        DiagnosticError::new(ErrorPhase::Runtime, token.line, token.column, message)
    }

    fn io_error(prefix: &str, error: std::io::Error) -> DiagnosticError {
        DiagnosticError::new(ErrorPhase::Runtime, 0, 0, format!("{prefix}: {error}"))
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

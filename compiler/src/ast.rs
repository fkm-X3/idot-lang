use crate::token::Token;
use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Token,
        args: Vec<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Value),
    Unary {
        op: Token,
        right: Box<Expr>,
    },
    Variable(Token),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expr(Expr),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
    Var {
        name: Token,
        initializer: Expr,
    },
}

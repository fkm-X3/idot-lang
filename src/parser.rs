use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Bool,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_: Type,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        type_: Type,
        init: Option<Expr>,
    },
    Expr(Expr),
    Return(Option<Expr>),
    If {
        cond: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Integer(i64),
    String(String),
    Bool(bool),
    Ident(String),
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Unary {
        op: UnOp,
        operand: Box<Expr>,
    },
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub errors: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::EOF)
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens.get(self.pos).cloned().unwrap_or(Token::EOF);
        self.pos += 1;
        token
    }

    fn expect(&mut self, expected: &Token) -> bool {
        if self.peek() == expected {
            self.advance();
            true
        } else {
            self.errors.push(format!(
                "expected {:?}, got {:?} at position {}",
                expected,
                self.peek(),
                self.pos
            ));
            false
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut functions = Vec::new();
        while self.peek() != &Token::EOF {
            functions.push(self.parse_function());
        }
        Program { functions }
    }

    fn parse_function(&mut self) -> Function {
        self.expect(&Token::Fn);
        let name = if let Token::Identifier(name) = self.advance() {
            name
        } else {
            self.errors.push(format!("expected function name, got {:?}", self.peek()));
            String::new()
        };

        self.expect(&Token::LParen);
        let mut params = Vec::new();
        if self.peek() != &Token::RParen {
            loop {
                let param_name = if let Token::Identifier(name) = self.advance() {
                    name
                } else {
                    self.errors.push(format!("expected parameter name, got {:?}", self.peek()));
                    String::new()
                };
                self.expect(&Token::Colon);
                let param_type = self.parse_type();
                params.push(Param { name: param_name, type_: param_type });
                if self.peek() != &Token::RParen {
                    self.expect(&Token::Comma);
                } else {
                    break;
                }
            }
        }
        self.expect(&Token::RParen);

        let return_type = if self.peek() == &Token::Arrow {
            self.advance();
            self.parse_type()
        } else {
            Type::Int
        };

        let body = self.parse_block();

        Function { name, params, return_type, body }
    }

    fn parse_type(&mut self) -> Type {
        match self.peek() {
            Token::Int => { self.advance(); Type::Int }
            Token::Bool => { self.advance(); Type::Bool }
            t => {
                self.errors.push(format!("expected type (int/bool), got {:?}", t));
                Type::Int
            }
        }
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        self.expect(&Token::LBrace);
        let mut stmts = Vec::new();
        while self.peek() != &Token::RBrace && self.peek() != &Token::EOF {
            stmts.push(self.parse_statement());
        }
        self.expect(&Token::RBrace);
        stmts
    }

    fn parse_statement(&mut self) -> Stmt {
        match self.peek() {
            Token::Let => self.parse_let(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::Return => self.parse_return(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let(&mut self) -> Stmt {
        self.expect(&Token::Let);
        let name = if let Token::Identifier(name) = self.advance() {
            name
        } else {
            self.errors.push(format!("expected identifier in let, got {:?}", self.peek()));
            String::new()
        };
        self.expect(&Token::Colon);
        let type_ = self.parse_type();
        let init = if self.peek() == &Token::Equal {
            self.advance();
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect(&Token::Semicolon);
        Stmt::Let { name, type_, init }
    }

    fn parse_if(&mut self) -> Stmt {
        self.expect(&Token::If);
        let cond = self.parse_expression();
        let then_block = self.parse_block();
        let else_block = if self.peek() == &Token::Else {
            self.advance();
            if self.peek() == &Token::If {
                Some(vec![self.parse_if()])
            } else {
                Some(self.parse_block())
            }
        } else {
            None
        };
        Stmt::If { cond, then_block, else_block }
    }

    fn parse_while(&mut self) -> Stmt {
        self.expect(&Token::While);
        let cond = self.parse_expression();
        let body = self.parse_block();
        Stmt::While { cond, body }
    }

    fn parse_return(&mut self) -> Stmt {
        self.expect(&Token::Return);
        if self.peek() == &Token::Semicolon {
            self.advance();
            Stmt::Return(None)
        } else {
            let expr = self.parse_expression();
            self.expect(&Token::Semicolon);
            Stmt::Return(Some(expr))
        }
    }

    fn parse_expr_stmt(&mut self) -> Stmt {
        let expr = self.parse_expression();
        self.expect(&Token::Semicolon);
        Stmt::Expr(expr)
    }

    fn parse_expression(&mut self) -> Expr {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Expr {
        let expr = self.parse_equality();
        if self.peek() == &Token::Equal {
            self.advance();
            if let Expr::Ident(name) = expr {
                let value = self.parse_assignment();
                return Expr::Assign { name, value: Box::new(value) };
            } else {
                self.errors.push("left side of assignment must be an identifier".into());
            }
        }
        expr
    }

    fn parse_equality(&mut self) -> Expr {
        let mut left = self.parse_comparison();
        while let Token::EqualEqual | Token::BangEqual = self.peek() {
            let op = match self.advance() {
                Token::EqualEqual => BinOp::Eq,
                Token::BangEqual => BinOp::Ne,
                _ => unreachable!(),
            };
            let right = self.parse_comparison();
            left = Expr::Binary { op, lhs: Box::new(left), rhs: Box::new(right) };
        }
        left
    }

    fn parse_comparison(&mut self) -> Expr {
        let mut left = self.parse_term();
        while let Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual = self.peek() {
            let op = match self.advance() {
                Token::Less => BinOp::Lt,
                Token::LessEqual => BinOp::Le,
                Token::Greater => BinOp::Gt,
                Token::GreaterEqual => BinOp::Ge,
                _ => unreachable!(),
            };
            let right = self.parse_term();
            left = Expr::Binary { op, lhs: Box::new(left), rhs: Box::new(right) };
        }
        left
    }

    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_factor();
        while let Token::Plus | Token::Minus = self.peek() {
            let op = match self.advance() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_factor();
            left = Expr::Binary { op, lhs: Box::new(left), rhs: Box::new(right) };
        }
        left
    }

    fn parse_factor(&mut self) -> Expr {
        let mut left = self.parse_unary();
        while let Token::Star | Token::Slash | Token::Percent = self.peek() {
            let op = match self.advance() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => unreachable!(),
            };
            let right = self.parse_unary();
            left = Expr::Binary { op, lhs: Box::new(left), rhs: Box::new(right) };
        }
        left
    }

    fn parse_unary(&mut self) -> Expr {
        if self.peek() == &Token::Minus {
            self.advance();
            let operand = self.parse_unary();
            return Expr::Unary { op: UnOp::Neg, operand: Box::new(operand) };
        }
        self.parse_primary()
    }

    fn parse_call_or_ident(&mut self, name: String) -> Expr {
        if self.peek() == &Token::LParen {
            self.advance();
            let mut args = Vec::new();
            if self.peek() != &Token::RParen {
                loop {
                    args.push(self.parse_expression());
                    if self.peek() != &Token::RParen {
                        self.expect(&Token::Comma);
                    } else {
                        break;
                    }
                }
            }
            self.expect(&Token::RParen);
            Expr::Call { name, args }
        } else {
            Expr::Ident(name)
        }
    }

    fn parse_primary(&mut self) -> Expr {
        match self.advance() {
            Token::Integer(n) => Expr::Integer(n),
            Token::String(s) => Expr::String(s),
            Token::True => Expr::Bool(true),
            Token::False => Expr::Bool(false),
            Token::Identifier(name) => {
                self.parse_call_or_ident(name)
            }
            Token::Print => {
                self.parse_call_or_ident("print".to_string())
            }
            Token::LParen => {
                let expr = self.parse_expression();
                self.expect(&Token::RParen);
                expr
            }
            t => {
                self.errors.push(format!("unexpected token {:?}", t));
                Expr::Integer(0)
            }
        }
    }
}

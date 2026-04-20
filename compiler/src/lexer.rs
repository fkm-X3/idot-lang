use std::error::Error;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Str(String),
    Var(String),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Unary(UnOp, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Eq, Neq, Lt, LtEq, Gt, GtEq,
}

#[derive(Debug, Clone)]
pub enum UnOp { Neg }

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Expr),
    Assign(String, Expr),
    Print(Expr),
    ExprStmt(Expr),
}

#[derive(Debug, Clone)]
enum Token {
    Ident(String),
    Number(i64),
    StringLit(String),
    Plus, Minus, Star, Slash,
    Equal, EqualEqual, BangEqual,
    Less, LessEqual, Greater, GreaterEqual,
    LeftParen, RightParen, Semicolon,
    Let, Print, If, Else,
    EOF,
}

fn is_ident_start(c: char) -> bool { c.is_ascii_alphabetic() || c == '_' }
fn is_ident_continue(c: char) -> bool { c.is_ascii_alphanumeric() || c == '_' }

fn tokenize(src: &str) -> Result<Vec<Token>, Box<dyn Error>> {
    let chars: Vec<char> = src.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() { i += 1; continue; }
        if is_ident_start(c) {
            let start = i; i += 1;
            while i < chars.len() && is_ident_continue(chars[i]) { i += 1; }
            let s: String = chars[start..i].iter().collect();
            match s.as_str() {
                "let" => tokens.push(Token::Let),
                "print" => tokens.push(Token::Print),
                "if" => tokens.push(Token::If),
                "else" => tokens.push(Token::Else),
                _ => tokens.push(Token::Ident(s)),
            }
            continue;
        }
        if c.is_ascii_digit() {
            let start = i; i += 1;
            while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
            let s: String = chars[start..i].iter().collect();
            let n = s.parse::<i64>()?;
            tokens.push(Token::Number(n));
            continue;
        }
        if c == '"' {
            i += 1; let start = i;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < chars.len() { i += 2; } else { i += 1; }
            }
            if i >= chars.len() { return Err("Unterminated string literal".into()); }
            let s: String = chars[start..i].iter().collect();
            i += 1;
            tokens.push(Token::StringLit(s));
            continue;
        }
        match c {
            '+' => { tokens.push(Token::Plus); i += 1; }
            '-' => { tokens.push(Token::Minus); i += 1; }
            '*' => { tokens.push(Token::Star); i += 1; }
            '/' => { tokens.push(Token::Slash); i += 1; }
            '(' => { tokens.push(Token::LeftParen); i += 1; }
            ')' => { tokens.push(Token::RightParen); i += 1; }
            ';' => { tokens.push(Token::Semicolon); i += 1; }
            '=' => { if i + 1 < chars.len() && chars[i+1] == '=' { tokens.push(Token::EqualEqual); i += 2; } else { tokens.push(Token::Equal); i += 1; } }
            '!' => { if i + 1 < chars.len() && chars[i+1] == '=' { tokens.push(Token::BangEqual); i += 2; } else { return Err(format!("Unexpected character '!'" ).into()); } }
            '<' => { if i + 1 < chars.len() && chars[i+1] == '=' { tokens.push(Token::LessEqual); i += 2; } else { tokens.push(Token::Less); i += 1; } }
            '>' => { if i + 1 < chars.len() && chars[i+1] == '=' { tokens.push(Token::GreaterEqual); i += 2; } else { tokens.push(Token::Greater); i += 1; } }
            _ => { return Err(format!("Unexpected character '{}'", c).into()); }
        }
    }
    tokens.push(Token::EOF);
    Ok(tokens)
}

struct Parser { tokens: Vec<Token>, pos: usize }

impl Parser {
    fn new(tokens: Vec<Token>) -> Self { Parser { tokens, pos: 0 } }
    fn peek(&self) -> &Token { &self.tokens[self.pos] }
    fn advance(&mut self) -> Token { let t = self.tokens[self.pos].clone(); self.pos += 1; t }
    fn at_eof(&self) -> bool { matches!(self.peek(), Token::EOF) }
    fn consume_semicolon(&mut self) -> Result<(), Box<dyn Error>> { match self.peek() { Token::Semicolon => { self.advance(); Ok(()) } _ => Err("Expected ';'".into()) }

    fn parse_program(&mut self) -> Result<Vec<Stmt>, Box<dyn Error>> {
        let mut out = Vec::new();
        while !self.at_eof() {
            if let Token::EOF = self.peek() { break; }
            out.push(self.parse_stmt()?);
        }
        Ok(out)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, Box<dyn Error>> {
        match self.peek().clone() {
            Token::Let => {
                self.advance();
                match self.advance() {
                    Token::Ident(name) => {
                        match self.peek() {
                            Token::Equal => { self.advance(); let expr = self.parse_expr()?; self.consume_semicolon()?; Ok(Stmt::Let(name, expr)) }
                            _ => Err("Expected '=' after identifier in let".into()),
                        }
                    }
                    _ => Err("Expected identifier after 'let'".into())
                }
            },
            Token::Print => {
                self.advance();
                let expr = self.parse_expr()?;
                self.consume_semicolon()?;
                Ok(Stmt::Print(expr))
            },
            Token::Ident(name) => {
                // lookahead for assignment
                if let Token::Equal = self.tokens[self.pos + 1] {
                    self.advance(); // ident
                    self.advance(); // '='
                    let expr = self.parse_expr()?;
                    self.consume_semicolon()?;
                    Ok(Stmt::Assign(name, expr))
                } else {
                    let expr = self.parse_expr()?;
                    self.consume_semicolon()?;
                    Ok(Stmt::ExprStmt(expr))
                }
            }
            _ => { let expr = self.parse_expr()?; self.consume_semicolon()?; Ok(Stmt::ExprStmt(expr)) }
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, Box<dyn Error>> { self.parse_equality() }

    fn parse_equality(&mut self) -> Result<Expr, Box<dyn Error>> {
        let mut expr = self.parse_comparison()?;
        loop {
            match self.peek() {
                Token::EqualEqual => { self.advance(); let right = self.parse_comparison()?; expr = Expr::Binary(Box::new(expr), BinOp::Eq, Box::new(right)); },
                Token::BangEqual => { self.advance(); let right = self.parse_comparison()?; expr = Expr::Binary(Box::new(expr), BinOp::Neq, Box::new(right)); },
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, Box<dyn Error>> {
        let mut expr = self.parse_term()?;
        loop {
            match self.peek() {
                Token::Less => { self.advance(); let right = self.parse_term()?; expr = Expr::Binary(Box::new(expr), BinOp::Lt, Box::new(right)); }
                Token::LessEqual => { self.advance(); let right = self.parse_term()?; expr = Expr::Binary(Box::new(expr), BinOp::LtEq, Box::new(right)); }
                Token::Greater => { self.advance(); let right = self.parse_term()?; expr = Expr::Binary(Box::new(expr), BinOp::Gt, Box::new(right)); }
                Token::GreaterEqual => { self.advance(); let right = self.parse_term()?; expr = Expr::Binary(Box::new(expr), BinOp::GtEq, Box::new(right)); }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, Box<dyn Error>> {
        let mut expr = self.parse_factor()?;
        loop {
            match self.peek() {
                Token::Plus => { self.advance(); let right = self.parse_factor()?; expr = Expr::Binary(Box::new(expr), BinOp::Add, Box::new(right)); }
                Token::Minus => { self.advance(); let right = self.parse_factor()?; expr = Expr::Binary(Box::new(expr), BinOp::Sub, Box::new(right)); }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, Box<dyn Error>> {
        let mut expr = self.parse_unary()?;
        loop {
            match self.peek() {
                Token::Star => { self.advance(); let right = self.parse_unary()?; expr = Expr::Binary(Box::new(expr), BinOp::Mul, Box::new(right)); }
                Token::Slash => { self.advance(); let right = self.parse_unary()?; expr = Expr::Binary(Box::new(expr), BinOp::Div, Box::new(right)); }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, Box<dyn Error>> {
        match self.peek() {
            Token::Minus => { self.advance(); let right = self.parse_unary()?; Ok(Expr::Unary(UnOp::Neg, Box::new(right))) }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, Box<dyn Error>> {
        match self.advance() {
            Token::Number(n) => Ok(Expr::Number(n)),
            Token::StringLit(s) => Ok(Expr::Str(s)),
            Token::Ident(name) => Ok(Expr::Var(name)),
            Token::LeftParen => { let expr = self.parse_expr()?; match self.advance() { Token::RightParen => Ok(expr), _ => Err("Expected ')'".into()), }
            }
            t => Err(format!("Unexpected token in expression: {:?}", t).into()),
        }
    }
}

pub fn tokenize_and_parse(src: &str) -> Result<Vec<Stmt>, Box<dyn Error>> {
    let tokens = tokenize(src)?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;
    Ok(program)
}

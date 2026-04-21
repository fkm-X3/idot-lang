use crate::ast::{Expr, Stmt};
use crate::diagnostics::{DiagnosticError, ErrorPhase, Result};
use crate::token::{Token, TokenType};
use crate::value::Value;

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>> {
    Parser::new(tokens).parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn parse_program(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt> {
        if self.match_types(&[TokenType::KeywordLet]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expected variable name after 'let'.")?;
        self.consume(TokenType::Equal, "Expected '=' after variable name.")?;
        let initializer = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration.",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_types(&[TokenType::KeywordIf]) {
            return self.if_statement();
        }
        if self.match_types(&[TokenType::KeywordPrint]) {
            return self.print_statement();
        }
        if self.match_types(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block(self.block()?));
        }
        self.expression_statement()
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after condition.")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_types(&[TokenType::KeywordElse]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after print expression.")?;
        Ok(Stmt::Print(expression))
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression.")?;
        Ok(Stmt::Expr(expression))
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expected '}' after block.")?;
        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expression = self.equality()?;
        if !self.match_types(&[TokenType::Equal]) {
            return Ok(expression);
        }

        let equals = self.previous().clone();
        let value = self.assignment()?;
        if let Expr::Variable(name) = expression {
            return Ok(Expr::Assign {
                name,
                value: Box::new(value),
            });
        }

        Err(DiagnosticError::new(
            ErrorPhase::Parse,
            equals.line,
            equals.column,
            "Invalid assignment target.",
        ))
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expression = self.comparison()?;
        while self.match_types(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let op = self.previous().clone();
            let right = self.comparison()?;
            expression = Expr::Binary {
                left: Box::new(expression),
                op,
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expression = self.term()?;
        while self.match_types(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = self.previous().clone();
            let right = self.term()?;
            expression = Expr::Binary {
                left: Box::new(expression),
                op,
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expression = self.factor()?;
        while self.match_types(&[TokenType::Minus, TokenType::Plus]) {
            let op = self.previous().clone();
            let right = self.factor()?;
            expression = Expr::Binary {
                left: Box::new(expression),
                op,
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expression = self.unary()?;
        while self.match_types(&[TokenType::Slash, TokenType::Star, TokenType::Percent]) {
            let op = self.previous().clone();
            let right = self.unary()?;
            expression = Expr::Binary {
                left: Box::new(expression),
                op,
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.match_types(&[TokenType::Bang, TokenType::Minus]) {
            let op = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                op,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.match_types(&[TokenType::KeywordFalse]) {
            return Ok(Expr::Literal(Value::Bool(false)));
        }
        if self.match_types(&[TokenType::KeywordTrue]) {
            return Ok(Expr::Literal(Value::Bool(true)));
        }
        if self.match_types(&[TokenType::KeywordNil]) {
            return Ok(Expr::Literal(Value::Nil));
        }
        if self.match_types(&[TokenType::Number]) {
            let token = self.previous().clone();
            let value = token.lexeme.parse::<f64>().map_err(|_| {
                DiagnosticError::new(
                    ErrorPhase::Parse,
                    token.line,
                    token.column,
                    "Invalid number literal.",
                )
            })?;
            return Ok(Expr::Literal(Value::Number(value)));
        }
        if self.match_types(&[TokenType::String]) {
            return Ok(Expr::Literal(Value::String(self.previous().lexeme.clone())));
        }
        if self.match_types(&[TokenType::Identifier]) {
            return Ok(Expr::Variable(self.previous().clone()));
        }
        if self.match_types(&[TokenType::LeftParen]) {
            let expression = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
            return Ok(Expr::Grouping(Box::new(expression)));
        }

        let token = self.peek();
        Err(DiagnosticError::new(
            ErrorPhase::Parse,
            token.line,
            token.column,
            "Expected expression.",
        ))
    }

    fn match_types(&mut self, kinds: &[TokenType]) -> bool {
        for kind in kinds {
            if self.check(*kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, kind: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().kind == kind
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenType::EndOfFile
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, kind: TokenType, message: &str) -> Result<Token> {
        if self.check(kind) {
            return Ok(self.advance().clone());
        }

        let token = self.peek();
        Err(DiagnosticError::new(
            ErrorPhase::Parse,
            token.line,
            token.column,
            message,
        ))
    }
}

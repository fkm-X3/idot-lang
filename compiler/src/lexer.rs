use crate::diagnostics::{DiagnosticError, ErrorPhase, Result};
use crate::token::{Token, TokenType};

struct Scanner {
    chars: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
    start_line: usize,
    start_column: usize,
}

impl Scanner {
    fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
            start_line: 1,
            start_column: 1,
        }
    }

    fn scan_tokens(mut self) -> Result<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.start_line = self.line;
            self.start_column = self.column;
            self.scan_token()?;
        }

        self.tokens.push(Token {
            kind: TokenType::EndOfFile,
            lexeme: String::new(),
            line: self.line,
            column: self.column,
        });
        Ok(self.tokens)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    fn advance(&mut self) -> char {
        let value = self.chars[self.current];
        self.current += 1;
        if value == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        value
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.chars[self.current] != expected {
            return false;
        }
        self.current += 1;
        self.column += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.chars.len() {
            '\0'
        } else {
            self.chars[self.current + 1]
        }
    }

    fn current_lexeme(&self) -> String {
        self.chars[self.start..self.current].iter().collect()
    }

    fn add_token(&mut self, kind: TokenType) {
        let lexeme = self.current_lexeme();
        self.tokens.push(Token {
            kind,
            lexeme,
            line: self.start_line,
            column: self.start_column,
        });
    }

    fn add_token_with_lexeme(&mut self, kind: TokenType, lexeme: String) {
        self.tokens.push(Token {
            kind,
            lexeme,
            line: self.start_line,
            column: self.start_column,
        });
    }

    fn scan_token(&mut self) -> Result<()> {
        let ch = self.advance();
        match ch {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '%' => self.add_token(TokenType::Percent),
            '!' => {
                let kind = if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(kind);
            }
            '=' => {
                let kind = if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(kind);
            }
            '<' => {
                let kind = if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(kind);
            }
            '>' => {
                let kind = if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(kind);
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            '"' => self.string()?,
            ' ' | '\r' | '\t' | '\n' => {}
            _ => {
                if ch.is_ascii_digit() {
                    self.number();
                } else if is_alpha(ch) {
                    self.identifier();
                } else {
                    return Err(DiagnosticError::new(
                        ErrorPhase::Lex,
                        self.start_line,
                        self.start_column,
                        format!("Unexpected character '{}'.", ch),
                    ));
                }
            }
        }
        Ok(())
    }

    fn string(&mut self) -> Result<()> {
        while self.peek() != '"' && !self.is_at_end() {
            self.advance();
        }

        if self.is_at_end() {
            return Err(DiagnosticError::new(
                ErrorPhase::Lex,
                self.start_line,
                self.start_column,
                "Unterminated string.",
            ));
        }

        self.advance();
        let lexeme: String = self.chars[self.start + 1..self.current - 1]
            .iter()
            .collect();
        self.add_token_with_lexeme(TokenType::String, lexeme);
        Ok(())
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.add_token(TokenType::Number);
    }

    fn identifier(&mut self) {
        while is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text = self.current_lexeme();
        let kind = match text.as_str() {
            "let" => TokenType::KeywordLet,
            "if" => TokenType::KeywordIf,
            "else" => TokenType::KeywordElse,
            "true" => TokenType::KeywordTrue,
            "false" => TokenType::KeywordFalse,
            "nil" => TokenType::KeywordNil,
            "print" => TokenType::KeywordPrint,
            _ => TokenType::Identifier,
        };

        self.add_token_with_lexeme(kind, text);
    }
}

fn is_alpha(value: char) -> bool {
    value.is_ascii_alphabetic() || value == '_'
}

fn is_alphanumeric(value: char) -> bool {
    value.is_ascii_alphanumeric() || value == '_'
}

pub fn scan_tokens(source: &str) -> Result<Vec<Token>> {
    Scanner::new(source).scan_tokens()
}

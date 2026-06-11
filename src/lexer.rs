#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Integer(i64),
    Identifier(String),

    Fn,
    Let,
    If,
    Else,
    While,
    Return,
    True,
    False,
    Int,
    Bool,
    Print,

    Plus,
    Minus,
    Star,
    Slash,

    Equal,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    Colon,
    Arrow,

    EOF,
    Illegal(String),
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        self.pos += 1;
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '/' && self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '/' {
                while let Some(c) = self.peek() {
                    if c == '\n' || c == '\r' {
                        break;
                    }
                    self.pos += 1;
                }
            }
            if !ch.is_ascii_whitespace() {
                break;
            }
            self.pos += 1;
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();
            match self.peek() {
                None => {
                    tokens.push(Token::EOF);
                    break;
                }
                Some(ch) if ch.is_ascii_digit() => {
                    let mut num = String::new();
                    while let Some(d) = self.peek() {
                        if d.is_ascii_digit() {
                            num.push(d);
                            self.pos += 1;
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Integer(num.parse().unwrap()));
                }
                Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {
                    let mut ident = String::new();
                    while let Some(a) = self.peek() {
                        if a.is_ascii_alphanumeric() || a == '_' {
                            ident.push(a);
                            self.pos += 1;
                        } else {
                            break;
                        }
                    }
                    tokens.push(match ident.as_str() {
                        "fn" => Token::Fn,
                        "let" => Token::Let,
                        "if" => Token::If,
                        "else" => Token::Else,
                        "while" => Token::While,
                        "return" => Token::Return,
                        "true" => Token::True,
                        "false" => Token::False,
                        "int" => Token::Int,
                        "bool" => Token::Bool,
                        "print" => Token::Print,
                        _ => Token::Identifier(ident),
                    });
                }
                Some('=') => {
                    self.pos += 1;
                    if self.peek() == Some('=') {
                        self.pos += 1;
                        tokens.push(Token::EqualEqual);
                    } else {
                        tokens.push(Token::Equal);
                    }
                }
                Some('!') => {
                    self.pos += 1;
                    if self.peek() == Some('=') {
                        self.pos += 1;
                        tokens.push(Token::BangEqual);
                    } else {
                        tokens.push(Token::Illegal("unexpected !".into()));
                    }
                }
                Some('<') => {
                    self.pos += 1;
                    if self.peek() == Some('=') {
                        self.pos += 1;
                        tokens.push(Token::LessEqual);
                    } else {
                        tokens.push(Token::Less);
                    }
                }
                Some('>') => {
                    self.pos += 1;
                    if self.peek() == Some('=') {
                        self.pos += 1;
                        tokens.push(Token::GreaterEqual);
                    } else {
                        tokens.push(Token::Greater);
                    }
                }
                Some('-') => {
                    self.pos += 1;
                    if self.peek() == Some('>') {
                        self.pos += 1;
                        tokens.push(Token::Arrow);
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
                Some('+') => { self.pos += 1; tokens.push(Token::Plus); }
                Some('*') => { self.pos += 1; tokens.push(Token::Star); }
                Some('/') => { self.pos += 1; tokens.push(Token::Slash); }
                Some('(') => { self.pos += 1; tokens.push(Token::LParen); }
                Some(')') => { self.pos += 1; tokens.push(Token::RParen); }
                Some('{') => { self.pos += 1; tokens.push(Token::LBrace); }
                Some('}') => { self.pos += 1; tokens.push(Token::RBrace); }
                Some(',') => { self.pos += 1; tokens.push(Token::Comma); }
                Some(';') => { self.pos += 1; tokens.push(Token::Semicolon); }
                Some(':') => { self.pos += 1; tokens.push(Token::Colon); }
                Some(c) => {
                    self.pos += 1;
                    tokens.push(Token::Illegal(format!("unexpected character: {}", c)));
                }
            }
        }

        tokens
    }
}

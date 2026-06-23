use crate::ast::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Fn,
    Const,
    Let,
    Mut,
    If,
    Else,
    For,
    While,
    Match,
    In,
    Return,
    Struct,
    Enum,
    Union,
    Import,
    Extern,
    Pub,
    True,
    False,
    Null,
    Break,
    Continue,

    // Literals
    IntLit(i64),
    FloatLit(f64),
    StrLit(String),
    CharLit(char),

    // Identifier
    Ident(String),

    // Arithmetic operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    // Assignment combinations
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,

    // Comparison
    EqEq,
    BangEq,
    Lt,
    Gt,
    LtEq,
    GtEq,

    // Logical
    AmpAmp,
    PipePipe,

    // Bitwise
    Amp,
    Pipe,
    Caret,
    Tilde,
    LtLt,
    GtGt,

    // Assignment / declaration
    Eq,
    Colon,
    ColonColon,
    ColonEq,

    // Arrows
    Arrow,
    FatArrow,

    // Ranges
    DotDot,
    DotDotEq,

    // Other
    Dot,
    Question,
    Bang,
    At,
    Hash,

    // Delimiters
    LParen,
    RParen,
    LBrack,
    RBrack,
    LBrace,
    RBrace,
    Semicolon,
    Comma,
    Underscore,

    // End of file
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos >= self.chars.len() {
                let span = self.pos..self.pos;
                tokens.push(Token { kind: TokenKind::Eof, span });
                return tokens;
            }

            let start = self.pos;
            let c = self.chars[self.pos];

            let kind = match c {
                // Single-line comment
                '/' if self.peek(1) == Some('/') => {
                    self.advance_while(|c| c != '\n');
                    continue;
                }
                // Block comment
                '/' if self.peek(1) == Some('*') => {
                    self.pos += 2;
                    loop {
                        if self.pos >= self.chars.len() {
                            panic!("Unterminated block comment starting at {}", start);
                        }
                        if self.chars[self.pos] == '*' && self.peek(1) == Some('/') {
                            self.pos += 2;
                            break;
                        }
                        self.pos += 1;
                    }
                    continue;
                }

                // String literals
                '"' => {
                    self.pos += 1;
                    let mut s = String::new();
                    loop {
                        if self.pos >= self.chars.len() {
                            panic!("Unterminated string literal starting at {}", start);
                        }
                        let ch = self.chars[self.pos];
                        if ch == '"' {
                            self.pos += 1;
                            break;
                        }
                        if ch == '\\' {
                            self.pos += 1;
                            if self.pos >= self.chars.len() {
                                panic!("Unterminated escape in string at {}", self.pos);
                            }
                            s.push(match self.chars[self.pos] {
                                'n' => '\n',
                                't' => '\t',
                                'r' => '\r',
                                '\\' => '\\',
                                '"' => '"',
                                '0' => '\0',
                                c => c,
                            });
                            self.pos += 1;
                        } else {
                            s.push(ch);
                            self.pos += 1;
                        }
                    }
                    TokenKind::StrLit(s)
                }

                // Char literals
                '\'' => {
                    self.pos += 1;
                    if self.pos >= self.chars.len() {
                        panic!("Unterminated char literal at {}", start);
                    }
                    let ch = if self.chars[self.pos] == '\\' {
                        self.pos += 1;
                        if self.pos >= self.chars.len() {
                            panic!("Unterminated escape in char at {}", self.pos);
                        }
                        let c = match self.chars[self.pos] {
                            'n' => '\n',
                            't' => '\t',
                            'r' => '\r',
                            '\\' => '\\',
                            '\'' => '\'',
                            '0' => '\0',
                            c => c,
                        };
                        self.pos += 1;
                        c
                    } else {
                        let c = self.chars[self.pos];
                        self.pos += 1;
                        c
                    };
                    if self.pos >= self.chars.len() || self.chars[self.pos] != '\'' {
                        panic!("Unterminated char literal at {}", start);
                    }
                    self.pos += 1;
                    TokenKind::CharLit(ch)
                }

                // Digits: int or float
                '0'..='9' => {
                    self.lex_number(start)
                }

                // Identifiers and keywords
                'a'..='z' | 'A'..='Z' | '_' => {
                    self.lex_ident_or_keyword(start)
                }

                // Operators and punctuation
                '+' => {
                    self.pos += 1;
                    if self.peek(0) == Some('=') { self.pos += 1; TokenKind::PlusEq }
                    else { TokenKind::Plus }
                }
                '-' => {
                    self.pos += 1;
                    match self.peek(0) {
                        Some('=') => { self.pos += 1; TokenKind::MinusEq }
                        Some('>') => { self.pos += 1; TokenKind::Arrow }
                        _ => TokenKind::Minus,
                    }
                }
                '*' => {
                    self.pos += 1;
                    if self.peek(0) == Some('=') { self.pos += 1; TokenKind::StarEq }
                    else { TokenKind::Star }
                }
                '/' => {
                    self.pos += 1;
                    if self.peek(0) == Some('=') { self.pos += 1; TokenKind::SlashEq }
                    else { TokenKind::Slash }
                }
                '%' => {
                    self.pos += 1;
                    if self.peek(0) == Some('=') { self.pos += 1; TokenKind::PercentEq }
                    else { TokenKind::Percent }
                }
                '=' => {
                    self.pos += 1;
                    match self.peek(0) {
                        Some('=') => { self.pos += 1; TokenKind::EqEq }
                        Some('>') => { self.pos += 1; TokenKind::FatArrow }
                        _ => TokenKind::Eq,
                    }
                }
                '!' => {
                    self.pos += 1;
                    if self.peek(0) == Some('=') { self.pos += 1; TokenKind::BangEq }
                    else { TokenKind::Bang }
                }
                '<' => {
                    self.pos += 1;
                    match self.peek(0) {
                        Some('=') => { self.pos += 1; TokenKind::LtEq }
                        Some('<') => { self.pos += 1; TokenKind::LtLt }
                        _ => TokenKind::Lt,
                    }
                }
                '>' => {
                    self.pos += 1;
                    match self.peek(0) {
                        Some('=') => { self.pos += 1; TokenKind::GtEq }
                        Some('>') => { self.pos += 1; TokenKind::GtGt }
                        _ => TokenKind::Gt,
                    }
                }
                ':' => {
                    self.pos += 1;
                    match self.peek(0) {
                        Some(':') => { self.pos += 1; TokenKind::ColonColon }
                        Some('=') => { self.pos += 1; TokenKind::ColonEq }
                        _ => TokenKind::Colon,
                    }
                }
                '&' => {
                    self.pos += 1;
                    if self.peek(0) == Some('&') { self.pos += 1; TokenKind::AmpAmp }
                    else { TokenKind::Amp }
                }
                '|' => {
                    self.pos += 1;
                    if self.peek(0) == Some('|') { self.pos += 1; TokenKind::PipePipe }
                    else { TokenKind::Pipe }
                }
                '^' => { self.pos += 1; TokenKind::Caret }
                '~' => { self.pos += 1; TokenKind::Tilde }
                '?' => { self.pos += 1; TokenKind::Question }
                '@' => { self.pos += 1; TokenKind::At }
                '#' => { self.pos += 1; TokenKind::Hash }
                '(' => { self.pos += 1; TokenKind::LParen }
                ')' => { self.pos += 1; TokenKind::RParen }
                '[' => { self.pos += 1; TokenKind::LBrack }
                ']' => { self.pos += 1; TokenKind::RBrack }
                '{' => { self.pos += 1; TokenKind::LBrace }
                '}' => { self.pos += 1; TokenKind::RBrace }
                ';' => { self.pos += 1; TokenKind::Semicolon }
                ',' => { self.pos += 1; TokenKind::Comma }
                '.' => {
                    self.pos += 1;
                    match self.peek(0) {
                        Some('.') => {
                            self.pos += 1;
                            if self.peek(0) == Some('=') { self.pos += 1; TokenKind::DotDotEq }
                            else { TokenKind::DotDot }
                        }
                        _ => TokenKind::Dot,
                    }
                }

                c => {
                    panic!("Unexpected character '{}' at position {}", c, self.pos);
                }
            };

            tokens.push(Token { kind, span: start..self.pos });
        }
    }

    fn lex_number(&mut self, start: usize) -> TokenKind {
        // Check for hex/octal/binary prefix
        if self.chars[self.pos] == '0' {
            let next = self.peek(1);
            if next == Some('x') || next == Some('X') {
                self.pos += 2;
                self.advance_while(|c| c.is_ascii_hexdigit());
                let s: String = self.chars[start + 2..self.pos].iter().collect();
                let val = i64::from_str_radix(&s, 16).unwrap_or_else(|_| panic!("Invalid hex literal at {}", start));
                return TokenKind::IntLit(val);
            }
            if next == Some('o') || next == Some('O') {
                self.pos += 2;
                self.advance_while(|c| c.is_ascii_digit() && c != '8' && c != '9');
                let s: String = self.chars[start + 2..self.pos].iter().collect();
                let val = i64::from_str_radix(&s, 8).unwrap_or_else(|_| panic!("Invalid octal literal at {}", start));
                return TokenKind::IntLit(val);
            }
            if next == Some('b') || next == Some('B') {
                self.pos += 2;
                self.advance_while(|c| c == '0' || c == '1');
                let s: String = self.chars[start + 2..self.pos].iter().collect();
                let val = i64::from_str_radix(&s, 2).unwrap_or_else(|_| panic!("Invalid binary literal at {}", start));
                return TokenKind::IntLit(val);
            }
        }

        // Decimal number
        self.advance_while(|c| c.is_ascii_digit());

        // Check if float
        if self.peek(0) == Some('.') && self.peek(1).map_or(false, |c| c.is_ascii_digit()) {
            self.pos += 1; // consume '.'
            self.advance_while(|c| c.is_ascii_digit());

            // Check for exponent
            if self.peek(0) == Some('e') || self.peek(0) == Some('E') {
                self.pos += 1;
                if self.peek(0) == Some('-') || self.peek(0) == Some('+') {
                    self.pos += 1;
                }
                self.advance_while(|c| c.is_ascii_digit());
            }

            let s: String = self.chars[start..self.pos].iter().collect();
            let val = s.parse::<f64>().unwrap_or_else(|_| panic!("Invalid float literal at {}", start));
            return TokenKind::FloatLit(val);
        }

        let s: String = self.chars[start..self.pos].iter().collect();
        let val = s.parse::<i64>().unwrap_or_else(|_| panic!("Invalid int literal at {}", start));
        TokenKind::IntLit(val)
    }

    fn lex_ident_or_keyword(&mut self, start: usize) -> TokenKind {
        self.advance_while(|c| c.is_alphanumeric() || c == '_');
        let s: String = self.chars[start..self.pos].iter().collect();
        match s.as_str() {
            "fn" => TokenKind::Fn,
            "const" => TokenKind::Const,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "for" => TokenKind::For,
            "while" => TokenKind::While,
            "match" => TokenKind::Match,
            "in" => TokenKind::In,
            "return" => TokenKind::Return,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "union" => TokenKind::Union,
            "import" => TokenKind::Import,
            "extern" => TokenKind::Extern,
            "pub" => TokenKind::Pub,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            _ => TokenKind::Ident(s),
        }
    }

    fn advance_while(&mut self, pred: impl Fn(char) -> bool) {
        while self.pos < self.chars.len() && pred(self.chars[self.pos]) {
            self.pos += 1;
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
                self.pos += 1;
            }
            // Skip line comments that start at this position after whitespace
            if self.pos + 1 < self.chars.len() && self.chars[self.pos] == '/' {
                if self.chars[self.pos + 1] == '/' {
                    self.advance_while(|c| c != '\n');
                    continue;
                }
                if self.chars[self.pos + 1] == '*' {
                    self.pos += 2;
                    loop {
                        if self.pos >= self.chars.len() {
                            return; // unterminated, but our tokenize handles it
                        }
                        if self.chars[self.pos] == '*' && self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '/' {
                            self.pos += 2;
                            break;
                        }
                        self.pos += 1;
                    }
                    continue;
                }
            }
            break;
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }
}

impl TokenKind {
    pub fn is_bin_op(&self) -> bool {
        matches!(self,
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash | TokenKind::Percent |
            TokenKind::EqEq | TokenKind::BangEq | TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq |
            TokenKind::AmpAmp | TokenKind::PipePipe |
            TokenKind::Amp | TokenKind::Pipe | TokenKind::Caret | TokenKind::LtLt | TokenKind::GtGt
        )
    }

    pub fn is_assign_op(&self) -> bool {
        matches!(self,
            TokenKind::Eq | TokenKind::PlusEq | TokenKind::MinusEq |
            TokenKind::StarEq | TokenKind::SlashEq | TokenKind::PercentEq
        )
    }
}

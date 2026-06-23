use crate::ast::*;
use crate::lexer::{Lexer, Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

macro_rules! unexpected_token {
    ($self:expr, $expected:expr) => {{
        let tok = &$self.tokens[$self.pos];
        panic!("Expected {:?} at position {:?}, found {:?}", $expected, tok.span, tok.kind);
    }};
}

impl Parser {
    pub fn new(source: &str) -> Self {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn peek_n(&self, n: usize) -> &TokenKind {
        &self.tokens[self.pos + n].kind
    }

    fn advance(&mut self) -> &TokenKind {
        let kind = &self.tokens[self.pos].kind;
        self.pos += 1;
        kind
    }

    fn skip(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, kind: TokenKind) {
        if *self.peek() == kind {
            self.skip();
        } else {
            unexpected_token!(self, kind);
        }
    }

    fn expect_ident(&mut self) -> String {
        match self.peek() {
            TokenKind::Ident(s) => {
                let s = s.clone();
                self.skip();
                s
            }
            _ => unexpected_token!(self, "identifier"),
        }
    }

    // === Program ===
    // program = decl*
    pub fn parse_program(&mut self) -> Vec<Decl> {
        let mut decls = Vec::new();
        while *self.peek() != TokenKind::Eof {
            decls.push(self.parse_decl());
        }
        decls
    }

    // === Declarations ===
    fn parse_decl(&mut self) -> Decl {
        match self.peek() {
            TokenKind::Import => self.parse_import(),
            TokenKind::Extern => self.parse_extern(),
            TokenKind::Pub => {
                self.skip();
                self.parse_pub_decl()
            }
            TokenKind::Fn => self.parse_fn_decl(false),
            TokenKind::Const => self.parse_const_decl(),
            TokenKind::Let => self.parse_let_decl(),
            TokenKind::Struct => self.parse_struct_decl(),
            TokenKind::Enum => self.parse_enum_decl(),
            TokenKind::Union => self.parse_union_decl(),
            _ => panic!("Unexpected token {:?} at start of declaration", self.peek()),
        }
    }

    fn parse_pub_decl(&mut self) -> Decl {
        match self.peek() {
            TokenKind::Fn => {
                let mut decl = self.parse_fn_decl(true);
                if let Decl::Fn(ref mut f) = decl {
                    f.pub_ = true;
                }
                decl
            }
            TokenKind::Const => self.parse_const_decl(),
            TokenKind::Let => self.parse_let_decl(),
            TokenKind::Struct => self.parse_struct_decl(),
            TokenKind::Enum => self.parse_enum_decl(),
            TokenKind::Union => self.parse_union_decl(),
            _ => unexpected_token!(self, "fn, const, let, struct, enum, or union after pub"),
        }
    }

    // === Import ===
    // import string ";"
    fn parse_import(&mut self) -> Decl {
        self.skip(); // import
        let path = match self.peek() {
            TokenKind::StrLit(s) => {
                let s = s.clone();
                self.skip();
                s
            }
            _ => unexpected_token!(self, "string literal"),
        };
        self.expect(TokenKind::Semicolon);
        Decl::Import(ImportDecl { path })
    }

    // === Extern ===
    // extern fn name(params) -> type ";"
    fn parse_extern(&mut self) -> Decl {
        self.parse_extern_decl()
    }

    fn parse_extern_decl(&mut self) -> Decl {
        self.skip_if(TokenKind::Extern);
        let (name, params, return_type) = self.parse_fn_sig();
        self.expect(TokenKind::Semicolon);
        Decl::Fn(FnDecl {
            name,
            params,
            return_type,
            resolved_ret_type: None,
            body: Block::empty(),
            is_extern: true,
            foreign_lib: None,
            pub_: true,
        })
    }

    // === Function Declaration ===
    // "pub"? "fn" ident "(" param_list ")" ("->" type_or_tuple)? block
    fn parse_fn_decl(&mut self, is_pub: bool) -> Decl {
        let (name, params, return_type) = self.parse_fn_sig();
        let body = self.parse_block();
        Decl::Fn(FnDecl {
            name,
            params,
            return_type,
            resolved_ret_type: None,
            body,
            is_extern: false,
            foreign_lib: None,
            pub_: is_pub,
        })
    }

    fn parse_fn_sig(&mut self) -> (String, Vec<Param>, Option<Type>) {
        self.skip(); // fn
        let name = self.expect_ident();
        self.expect(TokenKind::LParen);
        let params = self.parse_param_list();
        self.expect(TokenKind::RParen);
        let return_type = if *self.peek() == TokenKind::Arrow {
            self.skip();
            let t = self.parse_type_or_tuple();
            Some(t)
        } else {
            None
        };
        (name, params, return_type)
    }

    fn parse_param_list(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        if *self.peek() == TokenKind::RParen {
            return params;
        }
        loop {
            let name = self.expect_ident();
            self.expect(TokenKind::Colon);
            let type_ = self.parse_type();
            let default = if *self.peek() == TokenKind::Eq {
                self.skip();
                Some(self.parse_expr())
            } else {
                None
            };
            params.push(Param { name, type_, default, resolved_type: None });
            if *self.peek() == TokenKind::Comma {
                self.skip();
                if *self.peek() == TokenKind::RParen {
                    break;
                }
            } else {
                break;
            }
        }
        params
    }

    // === Let Declaration ===
    // "let" ["mut"] ident (":" type)? ("=" expr)? ";"
    fn parse_let_decl(&mut self) -> Decl {
        self.skip(); // let
        let mutable = if *self.peek() == TokenKind::Mut {
            self.skip();
            true
        } else {
            false
        };
        let name = self.expect_ident();
        let type_ = if *self.peek() == TokenKind::Colon {
            self.skip();
            Some(self.parse_type())
        } else {
            None
        };
        let init = if *self.peek() == TokenKind::Eq {
            self.skip();
            Some(self.parse_expr())
        } else {
            None
        };
        self.expect(TokenKind::Semicolon);
        Decl::Let(VarDecl { name, mutable, type_, init, resolved_type: None })
    }

    // === Const Declaration ===
    // "const" ident (":" type)? "=" expr ";"
    fn parse_const_decl(&mut self) -> Decl {
        self.skip(); // const
        let name = self.expect_ident();
        let type_ = if *self.peek() == TokenKind::Colon {
            self.skip();
            Some(self.parse_type())
        } else {
            None
        };
        self.expect(TokenKind::Eq);
        let init = self.parse_expr();
        self.expect(TokenKind::Semicolon);
        Decl::Const(ConstDecl { name, type_, init, resolved_type: None })
    }

    // === Struct Declaration ===
    // "pub"? "struct" ident "{" field_list "}"
    fn parse_struct_decl(&mut self) -> Decl {
        self.skip(); // struct
        let name = if matches!(self.peek(), TokenKind::Ident(_)) {
            self.expect_ident()
        } else {
            String::new()
        };
        self.expect(TokenKind::LBrace);
        let fields = self.parse_struct_fields();
        self.expect(TokenKind::RBrace);
        Decl::Struct(StructDecl { name, fields })
    }

    fn parse_struct_fields(&mut self) -> Vec<StructField> {
        let mut fields = Vec::new();
        while *self.peek() != TokenKind::RBrace && *self.peek() != TokenKind::Eof {
            let name = self.expect_ident();
            self.expect(TokenKind::Colon);
            let type_ = self.parse_type();
            let default = if *self.peek() == TokenKind::Eq {
                self.skip();
                Some(self.parse_expr())
            } else {
                None
            };
            fields.push(StructField { name, type_, default });
            if *self.peek() == TokenKind::Comma {
                self.skip();
            }
        }
        fields
    }

    // === Enum Declaration ===
    // "enum" ident ("(" type ")")? "{" variant_list "}"
    fn parse_enum_decl(&mut self) -> Decl {
        self.skip(); // enum
        let name = self.expect_ident();
        let backing_type = if *self.peek() == TokenKind::LParen {
            self.skip(); // (
            let t = self.parse_type();
            self.expect(TokenKind::RParen);
            Some(t)
        } else {
            None
        };
        self.expect(TokenKind::LBrace);
        let mut variants = Vec::new();
        while *self.peek() != TokenKind::RBrace && *self.peek() != TokenKind::Eof {
            let vname = self.expect_ident();
            let value = if *self.peek() == TokenKind::Eq {
                self.skip();
                Some(self.parse_expr())
            } else {
                None
            };
            variants.push(EnumVariant { name: vname, value });
            if *self.peek() == TokenKind::Comma {
                self.skip();
            }
        }
        self.expect(TokenKind::RBrace);
        Decl::Enum(EnumDecl { name, backing_type, variants })
    }

    // === Union Declaration ===
    // "union" ident ("(enum)")? "{" field_list "}"
    fn parse_union_decl(&mut self) -> Decl {
        self.skip(); // union
        let name = self.expect_ident();
        let tagged = if *self.peek() == TokenKind::LParen && self.peek_n(1) == &TokenKind::Enum {
            self.skip(); self.skip(); // ( enum
            self.expect(TokenKind::RParen);
            true
        } else {
            false
        };
        self.expect(TokenKind::LBrace);
        let fields = self.parse_struct_fields();
        self.expect(TokenKind::RBrace);
        Decl::Union(UnionDecl { name, tagged, fields })
    }

    // === Types ===
    // Type parsing is context-sensitive - used after ":" in declarations
    fn parse_type(&mut self) -> Type {
        self.parse_type_with_precedence()
    }

    fn parse_type_or_tuple(&mut self) -> Type {
        // For now, just a single type. Tuples: "(" type ("," type)+ ")"
        if *self.peek() == TokenKind::LParen {
            self.skip();
            let t = self.parse_type();
            self.expect(TokenKind::RParen);
            return t;
        }
        self.parse_type()
    }

    fn parse_type_with_precedence(&mut self) -> Type {
        // Prefix type operators: *T, ?T, !T, []T, [N]T
        match self.peek() {
            TokenKind::Star => {
                self.skip();
                if *self.peek() == TokenKind::Const {
                    self.skip();
                    Type::ConstPtr(Box::new(self.parse_type_with_precedence()))
                } else {
                    Type::Ptr(Box::new(self.parse_type_with_precedence()))
                }
            }
            TokenKind::Question => {
                self.skip();
                Type::Optional(Box::new(self.parse_type_with_precedence()))
            }
            TokenKind::Bang => {
                self.skip();
                Type::ErrorUnion(Box::new(self.parse_type_with_precedence()))
            }
            TokenKind::LBrack => {
                self.skip();
                if *self.peek() == TokenKind::RBrack {
                    // []T = slice
                    self.skip();
                    Type::Slice(Box::new(self.parse_type_with_precedence()))
                } else {
                    // [N]T = array
                    let size = match self.peek() {
                        TokenKind::IntLit(n) => {
                            let n = *n;
                            self.skip();
                            n as u64
                        }
                        _ => unexpected_token!(self, "integer literal for array size"),
                    };
                    self.expect(TokenKind::RBrack);
                    Type::Array(size, Box::new(self.parse_type_with_precedence()))
                }
            }
            TokenKind::Fn => {
                self.skip(); // fn
                self.expect(TokenKind::LParen);
                let mut params = Vec::new();
                if *self.peek() != TokenKind::RParen {
                    loop {
                        params.push(self.parse_type());
                        if *self.peek() == TokenKind::Comma {
                            self.skip();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RParen);
                let ret = if *self.peek() == TokenKind::Arrow {
                    self.skip();
                    Some(Box::new(self.parse_type()))
                } else {
                    None
                };
                Type::Fn(params, ret)
            }
            _ => {
                // Named type
                let name = self.expect_ident();
                Type::Named(name)
            }
        }
    }

    // === Expressions ===
    pub fn parse_expr(&mut self) -> Expr {
        self.parse_assign()
    }

    // Assignment: lhs = rhs
    fn parse_assign(&mut self) -> Expr {
        let lhs = self.parse_catch_orelse();
        if *self.peek() == TokenKind::Eq {
            self.skip();
            let rhs = self.parse_assign();
            Expr::Assign(Box::new(lhs), Box::new(rhs))
        } else {
            lhs
        }
    }

    // catch / orelse (lowest binary precedence)
    fn parse_catch_orelse(&mut self) -> Expr {
        let mut left = self.parse_or();
        loop {
            match self.peek() {
                TokenKind::Catch => {
                    self.skip();
                    let right = self.parse_catch_orelse();
                    left = Expr::Catch(Box::new(left), Box::new(right));
                }
                TokenKind::OrElse => {
                    self.skip();
                    let right = self.parse_catch_orelse();
                    left = Expr::OrElse(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        left
    }

    // Logical OR: ||
    fn parse_or(&mut self) -> Expr {
        let mut left = self.parse_and();
        while *self.peek() == TokenKind::PipePipe {
            self.skip();
            let right = self.parse_and();
            left = Expr::Binary(BinOp::Or, Box::new(left), Box::new(right));
        }
        left
    }

    // Logical AND: &&
    fn parse_and(&mut self) -> Expr {
        let mut left = self.parse_bit_or();
        while *self.peek() == TokenKind::AmpAmp {
            self.skip();
            let right = self.parse_bit_or();
            left = Expr::Binary(BinOp::And, Box::new(left), Box::new(right));
        }
        left
    }

    // Bitwise OR: |
    fn parse_bit_or(&mut self) -> Expr {
        let mut left = self.parse_bit_xor();
        while *self.peek() == TokenKind::Pipe {
            self.skip();
            let right = self.parse_bit_xor();
            left = Expr::Binary(BinOp::BitOr, Box::new(left), Box::new(right));
        }
        left
    }

    // Bitwise XOR: ^
    fn parse_bit_xor(&mut self) -> Expr {
        let mut left = self.parse_bit_and();
        while *self.peek() == TokenKind::Caret {
            self.skip();
            let right = self.parse_bit_and();
            left = Expr::Binary(BinOp::BitXor, Box::new(left), Box::new(right));
        }
        left
    }

    // Bitwise AND: &
    fn parse_bit_and(&mut self) -> Expr {
        let mut left = self.parse_equality();
        while *self.peek() == TokenKind::Amp {
            self.skip();
            let right = self.parse_equality();
            left = Expr::Binary(BinOp::BitAnd, Box::new(left), Box::new(right));
        }
        left
    }

    // Equality: ==, !=
    fn parse_equality(&mut self) -> Expr {
        let mut left = self.parse_comparison();
        loop {
            let op = match self.peek() {
                TokenKind::EqEq => BinOp::Eq,
                TokenKind::BangEq => BinOp::Ne,
                _ => break,
            };
            self.skip();
            let right = self.parse_comparison();
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        left
    }

    // Comparison: <, >, <=, >=
    fn parse_comparison(&mut self) -> Expr {
        let mut left = self.parse_shift();
        loop {
            let op = match self.peek() {
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::LtEq => BinOp::Le,
                TokenKind::GtEq => BinOp::Ge,
                _ => break,
            };
            self.skip();
            let right = self.parse_shift();
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        left
    }

    // Shift: <<, >>
    fn parse_shift(&mut self) -> Expr {
        let mut left = self.parse_term();
        loop {
            let op = match self.peek() {
                TokenKind::LtLt => BinOp::Shl,
                TokenKind::GtGt => BinOp::Shr,
                _ => break,
            };
            self.skip();
            let right = self.parse_term();
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        left
    }

    // Term: +, -
    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_factor();
        loop {
            let op = match self.peek() {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.skip();
            let right = self.parse_factor();
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        left
    }

    // Factor: *, /, %
    fn parse_factor(&mut self) -> Expr {
        let mut left = self.parse_unary();
        loop {
            let op = match self.peek() {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => break,
            };
            self.skip();
            let right = self.parse_unary();
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        left
    }

    // Unary: -x, !x, &x, *x, try x
    fn parse_unary(&mut self) -> Expr {
        match self.peek() {
            TokenKind::Minus => {
                self.skip();
                Expr::Unary(UnOp::Neg, Box::new(self.parse_unary()))
            }
            TokenKind::Bang => {
                self.skip();
                Expr::Unary(UnOp::Not, Box::new(self.parse_unary()))
            }
            TokenKind::Amp => {
                self.skip();
                Expr::Unary(UnOp::Addr, Box::new(self.parse_unary()))
            }
            TokenKind::Star => {
                self.skip();
                Expr::Unary(UnOp::Deref, Box::new(self.parse_unary()))
            }
            TokenKind::Try => {
                self.skip();
                Expr::Try(Box::new(self.parse_unary()))
            }
            _ => self.parse_postfix(),
        }
    }

    // Postfix: primary (call / index / field / slice)*
    fn parse_postfix(&mut self) -> Expr {
        let mut left = self.parse_primary();

        loop {
            match self.peek() {
                // Call: expr(args)
                TokenKind::LParen => {
                    self.skip();
                    let mut args = Vec::new();
                    if *self.peek() != TokenKind::RParen {
                        loop {
                            args.push(self.parse_expr());
                            if *self.peek() == TokenKind::Comma {
                                self.skip();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(TokenKind::RParen);
                    left = Expr::Call(Box::new(left), args);
                }
                // Index: expr[expr]
                TokenKind::LBrack => {
                    self.skip();
                    // Check for slice: expr[expr..expr] or expr[..expr] or expr[expr..]
                    if *self.peek() == TokenKind::DotDot || *self.peek() == TokenKind::DotDotEq {
                        // Slice with omitted start: expr[..end]
                        self.skip();
                        let end = if *self.peek() != TokenKind::RBrack {
                            Some(Box::new(self.parse_expr()))
                        } else {
                            None
                        };
                        self.expect(TokenKind::RBrack);
                        left = Expr::Slice(Box::new(left), None, end);
                    } else {
                        let start = self.parse_expr();
                        if *self.peek() == TokenKind::DotDot || *self.peek() == TokenKind::DotDotEq {
                            // Slice: expr[start..end]
                            let _inclusive = *self.peek() == TokenKind::DotDotEq;
                            self.skip();
                            let end = if *self.peek() != TokenKind::RBrack {
                                Some(Box::new(self.parse_expr()))
                            } else {
                                None
                            };
                            self.expect(TokenKind::RBrack);
                            left = Expr::Slice(Box::new(left), Some(Box::new(start)), end);
                        } else {
                            // Index: expr[index]
                            self.expect(TokenKind::RBrack);
                            left = Expr::Index(Box::new(left), Box::new(start));
                        }
                    }
                }
                // Field access: expr.field or pointer deref: expr.*
                TokenKind::Dot => {
                    self.skip();
                    if *self.peek() == TokenKind::Star {
                        self.skip();
                        left = Expr::Deref(Box::new(left));
                    } else {
                        let field = self.expect_ident();
                        left = Expr::Field(Box::new(left), field);
                    }
                }
                _ => break,
            }
        }

        left
    }

    // Primary: literals, ident, block, parenthesized, if, for, while, switch, struct init
    fn parse_primary(&mut self) -> Expr {
        match self.peek() {
            TokenKind::IntLit(n) => {
                let n = *n;
                self.skip();
                Expr::IntLit(n)
            }
            TokenKind::FloatLit(n) => {
                let n = *n;
                self.skip();
                Expr::FloatLit(n)
            }
            TokenKind::StrLit(s) => {
                let s = s.clone();
                self.skip();
                Expr::StrLit(s)
            }
            TokenKind::CharLit(c) => {
                let c = *c;
                self.skip();
                Expr::CharLit(c)
            }
            TokenKind::True => { self.skip(); Expr::BoolLit(true) }
            TokenKind::False => { self.skip(); Expr::BoolLit(false) }
            TokenKind::Null => { self.skip(); Expr::NullLit }

            TokenKind::Ident(s) => {
                let name = s.clone();
                self.skip();
                // Check for struct init: Foo{ ... }
                if *self.peek() == TokenKind::LBrace {
                    self.skip();
                    let mut fields = Vec::new();
                    if *self.peek() != TokenKind::RBrace {
                        loop {
                            let fname = self.expect_ident();
                            self.expect(TokenKind::Eq);
                            let val = self.parse_expr();
                            fields.push((fname, val));
                            if *self.peek() == TokenKind::Comma {
                                self.skip();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(TokenKind::RBrace);
                    Expr::StructInit(name, fields)
                } else {
                    Expr::Ident(name)
                }
            }

            TokenKind::LBrace => {
                let block = self.parse_block();
                Expr::Block(block)
            }

            TokenKind::LParen => {
                self.skip();
                let expr = self.parse_expr();
                self.expect(TokenKind::RParen);
                expr
            }

            TokenKind::If => self.parse_if_expr(),
            TokenKind::For => self.parse_for_expr(),
            TokenKind::While => self.parse_while_expr(),
            TokenKind::Match => self.parse_match_expr(),

            _ => unexpected_token!(self, "expression"),
        }
    }

    // === Blocks and Statements ===
    fn parse_block(&mut self) -> Block {
        self.expect(TokenKind::LBrace);
        let mut stmts = Vec::new();
        while *self.peek() != TokenKind::RBrace && *self.peek() != TokenKind::Eof {
            stmts.push(self.parse_stmt());
        }
        self.expect(TokenKind::RBrace);
        Block { stmts }
    }

    fn parse_stmt(&mut self) -> Stmt {
        match self.peek() {
            // Declaration statements
            TokenKind::Fn
            | TokenKind::Const
            | TokenKind::Let
            | TokenKind::Struct
            | TokenKind::Enum
            | TokenKind::Union
            | TokenKind::Import
            | TokenKind::Extern
            | TokenKind::Pub => Stmt::Decl(self.parse_decl()),

            // Control flow keywords as statements
            TokenKind::If => {
                let expr = self.parse_if_expr();
                Stmt::Expr(expr)
            }
            TokenKind::For => {
                let expr = self.parse_for_expr();
                Stmt::Expr(expr)
            }
            TokenKind::While => {
                let expr = self.parse_while_expr();
                Stmt::Expr(expr)
            }
            TokenKind::Match => {
                let expr = self.parse_match_expr();
                Stmt::Expr(expr)
            }

            // Defer
            TokenKind::Defer => {
                self.skip();
                let expr = self.parse_expr();
                // Semicolon is optional after defer (like Go/Zig)
                if *self.peek() == TokenKind::Semicolon {
                    self.skip();
                }
                Stmt::Defer(expr)
            }

            // Errdefer
            TokenKind::Errdefer => {
                self.skip();
                let expr = self.parse_expr();
                if *self.peek() == TokenKind::Semicolon {
                    self.skip();
                }
                Stmt::Errdefer(expr)
            }

            // Return
            TokenKind::Return => {
                self.skip();
                let expr = if *self.peek() != TokenKind::Semicolon {
                    Some(self.parse_expr())
                } else {
                    None
                };
                self.expect(TokenKind::Semicolon);
                Stmt::Return(expr)
            }

            TokenKind::Break => {
                self.skip();
                self.expect(TokenKind::Semicolon);
                Stmt::Break
            }

            TokenKind::Continue => {
                self.skip();
                self.expect(TokenKind::Semicolon);
                Stmt::Continue
            }

            // Blocks are expression statements
            TokenKind::LBrace => {
                let expr = self.parse_primary();
                Stmt::Expr(expr)
            }

            _ => {
                let expr = self.parse_expr();
                self.expect(TokenKind::Semicolon);
                Stmt::Expr(expr)
            }
        }
    }

    // === If Expression ===
    // if cond block ("else" (if | block))?
    fn parse_if_expr(&mut self) -> Expr {
        self.skip(); // if
        let cond = self.parse_expr();
        let then_block = self.parse_block();
        let else_branch = if *self.peek() == TokenKind::Else {
            self.skip();
            if *self.peek() == TokenKind::If {
                Some(Box::new(self.parse_if_expr()))
            } else {
                let block = self.parse_block();
                Some(Box::new(Expr::Block(block)))
            }
        } else {
            None
        };
        Expr::If(Box::new(cond), then_block, else_branch)
    }

    // === For Expression ===
    // for item in expr block
    fn parse_for_expr(&mut self) -> Expr {
        self.skip(); // for
        let item = self.expect_ident();
        self.expect(TokenKind::In);
        // Parse iterable expression, but don't let parse_primary consume
        // a following { as a struct init — it belongs to the for loop body.
        let iterable = if matches!(self.peek(), TokenKind::Ident(_))
            && self.peek_n(1) == &TokenKind::LBrace
        {
            let name = self.expect_ident();
            Expr::Ident(name)
        } else {
            self.parse_expr()
        };
        let body = self.parse_block();
        Expr::For(Box::new(iterable), Some(item), None, body)
    }

    // === While Expression ===
    // while cond block
    fn parse_while_expr(&mut self) -> Expr {
        self.skip(); // while
        let cond = self.parse_expr();
        let body = self.parse_block();
        Expr::While(Box::new(cond), body)
    }

    // === Match Expression ===
    // match expr "{" (pattern "=>" block (",")?)* "}"
    fn parse_match_expr(&mut self) -> Expr {
        self.skip(); // match
        let expr = self.parse_expr();
        self.expect(TokenKind::LBrace);
        let mut arms = Vec::new();
        let mut wildcard_arm = None;
        while *self.peek() != TokenKind::RBrace && *self.peek() != TokenKind::Eof {
            if matches!(self.peek(), TokenKind::Underscore) {
                self.skip();
                self.expect(TokenKind::FatArrow);
                wildcard_arm = Some(self.parse_block());
                if *self.peek() == TokenKind::Comma {
                    self.skip();
                }
                break;
            }
            let patterns = self.parse_match_patterns();
            self.expect(TokenKind::FatArrow);
            let body = self.parse_block();
            arms.push(MatchArm { patterns, body });
            if *self.peek() == TokenKind::Comma {
                self.skip();
            }
        }
        self.expect(TokenKind::RBrace);
        Expr::Match(Box::new(expr), arms, wildcard_arm)
    }

    fn parse_match_patterns(&mut self) -> Vec<MatchPattern> {
        let mut patterns = Vec::new();
        loop {
            let pattern = if matches!(self.peek(), TokenKind::Underscore) {
                patterns.push(MatchPattern::Wildcard);
                return patterns;
            } else {
                self.parse_pattern()
            };
            patterns.push(pattern);
            if *self.peek() == TokenKind::Comma {
                self.skip();
                if *self.peek() == TokenKind::FatArrow {
                    break;
                }
            } else {
                break;
            }
        }
        patterns
    }

    fn parse_pattern(&mut self) -> MatchPattern {
        let start = self.parse_expr();
        if *self.peek() == TokenKind::DotDot || *self.peek() == TokenKind::DotDotEq {
            self.skip();
            let end = self.parse_expr();
            MatchPattern::Range(start, end)
        } else {
            MatchPattern::Expr(start)
        }
    }

    // === Helpers ===
    fn skip_if(&mut self, kind: TokenKind) -> bool {
        if *self.peek() == kind {
            self.skip();
            true
        } else {
            false
        }
    }

    // === Entry point ===
    pub fn parse(&mut self) -> Vec<Decl> {
        self.parse_program()
    }
}

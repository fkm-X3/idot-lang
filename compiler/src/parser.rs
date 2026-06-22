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
            TokenKind::Foreign => self.parse_foreign(),
            TokenKind::Extern => self.parse_extern(),
            TokenKind::Pub => {
                self.skip();
                self.parse_pub_decl()
            }
            TokenKind::Fn => self.parse_fn_decl(false),
            TokenKind::Const => self.parse_const_decl(),
            TokenKind::Struct => self.parse_struct_decl(),
            TokenKind::Enum => self.parse_enum_decl(),
            TokenKind::Union => self.parse_union_decl(),
            TokenKind::Var => self.parse_var_decl(),
            // Odin-style constant: ident "::" expr
            TokenKind::Ident(_) if self.peek_n(1) == &TokenKind::ColonColon => {
                self.parse_odin_const()
            }
            // Variable with type: ident ":" type "=" ...
            TokenKind::Ident(_) if self.peek_n(1) == &TokenKind::Colon => {
                self.parse_var_decl_typed()
            }
            // Variable with inference: ident ":=" expr
            TokenKind::Ident(_) if self.peek_n(1) == &TokenKind::ColonEq => {
                self.parse_var_decl_inferred()
            }
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
            TokenKind::Struct => self.parse_struct_decl(),
            TokenKind::Enum => self.parse_enum_decl(),
            TokenKind::Union => self.parse_union_decl(),
            _ => unexpected_token!(self, "fn, const, struct, enum, or union after pub"),
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

    // === Foreign ===
    // foreign import ident string ";"
    // foreign ident "{" decl* "}"
    fn parse_foreign(&mut self) -> Decl {
        self.skip(); // foreign
        if *self.peek() == TokenKind::Import {
            self.skip(); // import
            let name = self.expect_ident();
            let path = match self.peek() {
                TokenKind::StrLit(s) => {
                    let s = s.clone();
                    self.skip();
                    Some(s)
                }
                _ => None,
            };
            self.expect(TokenKind::Semicolon);
            return Decl::Foreign(ForeignDecl {
                lib_name: name,
                lib_path: path,
                declarations: vec![],
            });
        }
        let name = self.expect_ident();
        self.expect(TokenKind::LBrace);
        let mut decls = Vec::new();
        while *self.peek() != TokenKind::RBrace && *self.peek() != TokenKind::Eof {
            decls.push(self.parse_extern_decl());
        }
        self.expect(TokenKind::RBrace);
        Decl::Foreign(ForeignDecl {
            lib_name: name,
            lib_path: None,
            declarations: decls,
        })
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

    // === Const Declaration ===
    // "const" ident (":" type)? ("=" | ":=") expr ";"
    fn parse_const_decl(&mut self) -> Decl {
        self.skip(); // const
        let name = self.expect_ident();
        let type_ = if *self.peek() == TokenKind::Colon {
            self.skip();
            Some(self.parse_type())
        } else {
            None
        };

        // Handle `const ident := expr` (inferred const)
        if *self.peek() == TokenKind::ColonEq {
            self.skip();
            let init = self.parse_expr();
            self.expect(TokenKind::Semicolon);
            return Decl::Const(ConstDecl { name, type_: None, init, resolved_type: None });
        }

        self.expect(TokenKind::Eq);
        let init = self.parse_expr();
        self.expect(TokenKind::Semicolon);
        Decl::Const(ConstDecl { name, type_, init, resolved_type: None })
    }

    // Odin-style: ident "::" expr ";"
    fn parse_odin_const(&mut self) -> Decl {
        let name = self.expect_ident();
        self.skip(); // ::

        // Check for type declarations: ident :: struct/enum/union { }
        match self.peek() {
            TokenKind::Struct => {
                self.skip();
                self.expect(TokenKind::LBrace);
                let fields = self.parse_struct_fields();
                self.expect(TokenKind::RBrace);
                return Decl::Struct(StructDecl { name, fields });
            }
            TokenKind::Enum => {
                self.skip();
                let backing_type = if *self.peek() == TokenKind::LParen {
                    self.skip();
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
                return Decl::Enum(EnumDecl { name, backing_type, variants });
            }
            TokenKind::Union => {
                self.skip();
                let tagged = if *self.peek() == TokenKind::LParen
                    && self.peek_n(1) == &TokenKind::Enum
                {
                    self.skip();
                    self.skip();
                    self.expect(TokenKind::RParen);
                    true
                } else {
                    false
                };
                self.expect(TokenKind::LBrace);
                let fields = self.parse_struct_fields();
                self.expect(TokenKind::RBrace);
                return Decl::Union(UnionDecl { name, tagged, fields });
            }
            _ => {}
        }

        let init = self.parse_expr();
        self.expect(TokenKind::Semicolon);
        Decl::Const(ConstDecl { name, type_: None, init, resolved_type: None })
    }

    // === Var Declaration ===
    // "var"? ident ":" type ("=" expr)? ";"
    fn parse_var_decl(&mut self) -> Decl {
        self.skip(); // var
        self.parse_var_decl_typed_mut(true)
    }

    fn parse_var_decl_typed(&mut self) -> Decl {
        self.parse_var_decl_typed_mut(true)
    }

    fn parse_var_decl_typed_mut(&mut self, mutable: bool) -> Decl {
        let name = self.expect_ident();
        self.expect(TokenKind::Colon);
        let type_ = self.parse_type();
        let init = if *self.peek() == TokenKind::Eq {
            self.skip();
            Some(self.parse_expr())
        } else {
            None
        };
        self.expect(TokenKind::Semicolon);
        Decl::Var(VarDecl { name, mutable, type_: Some(type_), init, resolved_type: None })
    }

    // ident ":=" expr ";"
    fn parse_var_decl_inferred(&mut self) -> Decl {
        let name = self.expect_ident();
        self.skip(); // :=
        let init = Some(self.parse_expr());
        self.expect(TokenKind::Semicolon);
        Decl::Var(VarDecl { name, mutable: true, type_: None, init, resolved_type: None })
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
    // ident "::"? "enum" ("(" type ")")? "{" variant_list "}"
    fn parse_enum_decl(&mut self) -> Decl {
        let name = if matches!(self.peek(), TokenKind::Ident(_)) {
            let n = self.expect_ident();
            if *self.peek() == TokenKind::ColonColon {
                self.skip();
            }
            n
        } else {
            self.skip(); // enum
            String::new()
        };
        if *self.peek() == TokenKind::Enum {
            self.skip();
        }
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
    fn parse_union_decl(&mut self) -> Decl {
        let name = if matches!(self.peek(), TokenKind::Ident(_)) {
            let n = self.expect_ident();
            if *self.peek() == TokenKind::ColonColon {
                self.skip();
            }
            n
        } else {
            self.skip(); // union
            String::new()
        };
        if *self.peek() == TokenKind::Union {
            self.skip();
        }
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
        let lhs = self.parse_catch();
        if *self.peek() == TokenKind::Eq {
            self.skip();
            let rhs = self.parse_assign();
            Expr::Assign(Box::new(lhs), Box::new(rhs))
        } else {
            lhs
        }
    }

    // Catch: lhs catch |err| { ... }
    fn parse_catch(&mut self) -> Expr {
        let lhs = self.parse_or();
        if *self.peek() == TokenKind::Catch {
            self.skip();
            let var = if *self.peek() == TokenKind::Pipe {
                self.skip();
                let v = self.expect_ident();
                self.expect(TokenKind::Pipe);
                Some(v)
            } else {
                None
            };
            let body = self.parse_block();
            Expr::Catch(Box::new(lhs), var, body)
        } else {
            lhs
        }
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
                // Field access: expr.field
                TokenKind::Dot => {
                    self.skip();
                    let field = self.expect_ident();
                    left = Expr::Field(Box::new(left), field);
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
            TokenKind::Undefined => { self.skip(); Expr::Undefined }

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
            TokenKind::Switch => self.parse_switch_expr(),
            TokenKind::Comptime => self.parse_comptime_expr(),
            TokenKind::When => self.parse_when_expr(),

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
            | TokenKind::Struct
            | TokenKind::Enum
            | TokenKind::Union
            | TokenKind::Import
            | TokenKind::Foreign
            | TokenKind::Extern
            | TokenKind::Var
            | TokenKind::Pub => Stmt::Decl(self.parse_decl()),

            // Ident could be a declaration (ident: type, ident ::, ident :=)
            TokenKind::Ident(_) => {
                // Peek ahead to check if this is a declaration or expression
                if self.peek_n(1) == &TokenKind::ColonColon {
                    // Odin const: ident :: expr
                    return Stmt::Decl(self.parse_decl());
                }
                if self.peek_n(1) == &TokenKind::Colon {
                    // Could be var decl with type or expression
                    return Stmt::Decl(self.parse_decl());
                }
                if self.peek_n(1) == &TokenKind::ColonEq {
                    // Inferred var: ident := expr
                    return Stmt::Decl(self.parse_decl());
                }
                // Expression statement
                let expr = self.parse_expr();
                self.expect(TokenKind::Semicolon);
                Stmt::Expr(expr)
            }

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
            TokenKind::Switch => {
                let expr = self.parse_switch_expr();
                Stmt::Expr(expr)
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

            // Defer / errdefer
            TokenKind::Defer => {
                self.skip();
                let expr = self.parse_expr();
                self.expect(TokenKind::Semicolon);
                Stmt::Defer(expr)
            }
            TokenKind::Errdefer => {
                self.skip();
                let expr = self.parse_expr();
                self.expect(TokenKind::Semicolon);
                Stmt::Errdefer(expr)
            }

            // Comptime / when as statements
            TokenKind::Comptime => {
                let expr = self.parse_comptime_expr();
                Stmt::Expr(expr)
            }
            TokenKind::When => {
                let expr = self.parse_when_expr();
                Stmt::Expr(expr)
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
    // for expr |item| block
    // for expr |item, index| block
    fn parse_for_expr(&mut self) -> Expr {
        self.skip(); // for
        // Parse the iterable expression at a precedence below bitwise OR/AND/logical OR
        // to avoid conflict with the |item| capture syntax
        let iterable = self.parse_bit_xor();
        self.expect(TokenKind::Pipe);
        let item = self.expect_ident();
        let index = if *self.peek() == TokenKind::Comma {
            self.skip();
            Some(self.expect_ident())
        } else {
            None
        };
        self.expect(TokenKind::Pipe);
        let body = self.parse_block();
        Expr::For(Box::new(iterable), Some(item), index, body)
    }

    // === While Expression ===
    // while cond block
    fn parse_while_expr(&mut self) -> Expr {
        self.skip(); // while
        let cond = self.parse_expr();
        let body = self.parse_block();
        Expr::While(Box::new(cond), body)
    }

    // === Switch Expression ===
    // switch expr "{" (pattern "=>" block (",")?)* "}"
    fn parse_switch_expr(&mut self) -> Expr {
        self.skip(); // switch
        let expr = self.parse_expr();
        self.expect(TokenKind::LBrace);
        let mut arms = Vec::new();
        let mut else_arm = None;
        while *self.peek() != TokenKind::RBrace && *self.peek() != TokenKind::Eof {
            if *self.peek() == TokenKind::Else {
                self.skip();
                self.expect(TokenKind::FatArrow);
                else_arm = Some(self.parse_block());
                // Allow trailing comma
                if *self.peek() == TokenKind::Comma {
                    self.skip();
                }
                break;
            }
            let patterns = self.parse_switch_patterns();
            self.expect(TokenKind::FatArrow);
            let body = self.parse_block();
            arms.push(SwitchArm { patterns, body });
            if *self.peek() == TokenKind::Comma {
                self.skip();
            }
        }
        self.expect(TokenKind::RBrace);
        Expr::Switch(Box::new(expr), arms, else_arm)
    }

    fn parse_switch_patterns(&mut self) -> Vec<SwitchPattern> {
        let mut patterns = Vec::new();
        loop {
            let pattern = if *self.peek() == TokenKind::Else {
                patterns.push(SwitchPattern::Else);
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

    fn parse_pattern(&mut self) -> SwitchPattern {
        let start = self.parse_expr();
        if *self.peek() == TokenKind::DotDot || *self.peek() == TokenKind::DotDotEq {
            self.skip();
            let end = self.parse_expr();
            SwitchPattern::Range(start, end)
        } else {
            SwitchPattern::Expr(start)
        }
    }

    // === Comptime ===
    // comptime block
    fn parse_comptime_expr(&mut self) -> Expr {
        self.skip(); // comptime
        let block = self.parse_block();
        Expr::Comptime(block)
    }

    // === When ===
    // when cond block ("else" block)?
    fn parse_when_expr(&mut self) -> Expr {
        self.skip(); // when
        let cond = self.parse_expr();
        let then_block = self.parse_block();
        let else_block = if *self.peek() == TokenKind::Else {
            self.skip();
            Some(self.parse_block())
        } else {
            None
        };
        Expr::When(Box::new(cond), then_block, else_block)
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

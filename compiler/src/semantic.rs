use std::collections::HashMap;
use crate::ast::*;

#[derive(Debug, Clone)]
struct Symbol {
    kind: SymbolKind,
    type_: TypeVal,
}

#[derive(Debug, Clone)]
enum SymbolKind {
    Variable { mutable: bool },
    Function,
    Type,
}

#[derive(Debug)]
struct Scope {
    symbols: HashMap<String, Symbol>,
}

#[derive(Debug)]
pub struct SemanticAnalyzer {
    scopes: Vec<Scope>,
    pub errors: Vec<String>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = SemanticAnalyzer {
            scopes: vec![],
            errors: vec![],
        };
        analyzer.push_scope();

        // Register built-in types in the global scope
        let builtins = [
            "void", "bool", "i8", "i16", "i32", "i64",
            "u8", "u16", "u32", "u64", "isize", "usize",
            "f32", "f64", "string",
        ];
        for name in &builtins {
            let tv = analyzer.resolve_named_type(name);
            analyzer.scopes[0].symbols.insert(
                name.to_string(),
                Symbol {
                    kind: SymbolKind::Type,
                    type_: tv,
                },
            );
        }

        analyzer
    }

    pub fn analyze(&mut self, program: &mut [Decl]) -> Result<(), Vec<String>> {
        // Phase 1: collect top-level declarations
        self.collect_top_level(program);

        if !self.errors.is_empty() {
            return Err(std::mem::take(&mut self.errors));
        }

        // Phase 2: resolve function bodies
        for decl in program.iter_mut() {
            if let Decl::Fn(f) = decl {
                self.analyze_fn_decl(f);
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    // === Scope ===

    fn push_scope(&mut self) {
        self.scopes.push(Scope {
            symbols: HashMap::new(),
        });
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.symbols.get(name) {
                return Some(sym);
            }
        }
        None
    }

    fn lookup_type_val(&self, name: &str) -> Option<TypeVal> {
        self.lookup(name).map(|s| s.type_.clone())
    }

    fn add_symbol(&mut self, name: String, kind: SymbolKind, type_: TypeVal) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.symbols.contains_key(&name) {
                self.errors
                    .push(format!("Duplicate declaration: '{}'", name));
                return;
            }
            scope
                .symbols
                .insert(name, Symbol { kind, type_ });
        }
    }

    // === Phase 1: Collect top-level ===

    fn collect_top_level(&mut self, program: &[Decl]) {
        for decl in program {
            match decl {
                Decl::Fn(f) => {
                    let params: Vec<TypeVal> = f
                        .params
                        .iter()
                        .map(|p| self.type_from_ast(&p.type_))
                        .collect();
                    let ret = f
                        .return_type
                        .as_ref()
                        .map(|t| self.type_from_ast(t))
                        .unwrap_or(TypeVal::Void);
                    self.add_symbol(
                        f.name.clone(),
                        SymbolKind::Function,
                        TypeVal::Fn(params, Some(Box::new(ret))),
                    );
                }
                Decl::Struct(s) => {
                    let fields: Vec<(String, TypeVal)> = s
                        .fields
                        .iter()
                        .map(|f| (f.name.clone(), self.type_from_ast(&f.type_)))
                        .collect();
                    self.add_symbol(
                        s.name.clone(),
                        SymbolKind::Type,
                        TypeVal::Struct(fields),
                    );
                }
                Decl::Enum(e) => {
                    self.add_symbol(
                        e.name.clone(),
                        SymbolKind::Type,
                        TypeVal::Named(e.name.clone()),
                    );
                }
                Decl::Import(_) | Decl::Foreign(_) | Decl::Union(_) => {
                    // Not yet handled in semantic analysis
                }
                _ => {}
            }
        }
    }

    // === Phase 2: Analyze function bodies ===

    fn analyze_fn_decl(&mut self, f: &mut FnDecl) {
        self.push_scope();

        // Resolve return type
        f.resolved_ret_type = f
            .return_type
            .as_ref()
            .map(|t| self.type_from_ast(t))
            .or(Some(TypeVal::Void));

        // Add parameters to scope
        for param in f.params.iter_mut() {
            let tv = self.type_from_ast(&param.type_);
            param.resolved_type = Some(tv.clone());
            self.add_symbol(
                param.name.clone(),
                SymbolKind::Variable { mutable: false },
                tv,
            );
        }

        // Analyze body
        self.analyze_block(&mut f.body);

        self.pop_scope();
    }

    fn analyze_block(&mut self, block: &mut Block) -> Option<TypeVal> {
        let mut last_type = None;
        for stmt in block.stmts.iter_mut() {
            last_type = self.analyze_stmt(stmt);
        }
        last_type
    }

    fn analyze_stmt(&mut self, stmt: &mut Stmt) -> Option<TypeVal> {
        match stmt {
            Stmt::Decl(decl) => self.analyze_decl(decl),
            Stmt::Expr(expr) => self.type_of_expr(expr),
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.type_of_expr(e);
                }
                None
            }
            Stmt::Break | Stmt::Continue => None,
            Stmt::Defer(expr) => {
                self.type_of_expr(expr);
                None
            }
            Stmt::Errdefer(expr) => {
                self.type_of_expr(expr);
                None
            }
        }
    }

    fn analyze_decl(&mut self, decl: &mut Decl) -> Option<TypeVal> {
        match decl {
            Decl::Var(v) => {
                let declared = v
                    .type_
                    .as_ref()
                    .map(|t| self.type_from_ast(t));

                if let Some(init) = &mut v.init {
                    let init_type = self.type_of_expr(init);
                    v.resolved_type = declared.clone().or(init_type);

                    if let (Some(ref dt), Some(ref it)) = (&declared, &v.resolved_type) {
                        if dt != it {
                            self.errors.push(format!(
                                "Type mismatch in '{}': declared {:?}, got {:?}",
                                v.name, dt, it
                            ));
                        }
                    }
                } else {
                    v.resolved_type = declared;
                }

                if let Some(ref tv) = v.resolved_type {
                    self.add_symbol(
                        v.name.clone(),
                        SymbolKind::Variable {
                            mutable: v.mutable,
                        },
                        tv.clone(),
                    );
                }
                v.resolved_type.clone()
            }

            Decl::Const(c) => {
                let declared = c
                    .type_
                    .as_ref()
                    .map(|t| self.type_from_ast(t));

                let init_type = self.type_of_expr(&mut c.init);
                c.resolved_type = declared.or(init_type);

                if let Some(ref tv) = c.resolved_type {
                    self.add_symbol(
                        c.name.clone(),
                        SymbolKind::Variable { mutable: false },
                        tv.clone(),
                    );
                }
                c.resolved_type.clone()
            }

            Decl::Fn(f) => {
                self.analyze_fn_decl(f);
                None
            }

            _ => None,
        }
    }

    // === Expression type inference ===

    fn type_of_expr(&mut self, expr: &mut Expr) -> Option<TypeVal> {
        match expr {
            Expr::IntLit(n) => {
                if *n >= i32::MIN as i64 && *n <= i32::MAX as i64 {
                    Some(TypeVal::Int(IntSize::I32))
                } else {
                    Some(TypeVal::Int(IntSize::I64))
                }
            }
            Expr::FloatLit(_) => Some(TypeVal::Float(FloatSize::F64)),
            Expr::BoolLit(_) => Some(TypeVal::Bool),
            Expr::StrLit(_) => Some(TypeVal::Slice(Box::new(TypeVal::Int(IntSize::U8)))),
            Expr::CharLit(_) => Some(TypeVal::Int(IntSize::I32)),
            Expr::NullLit => Some(TypeVal::NullablePtr(Box::new(TypeVal::Void))),
            Expr::Undefined => None,

            Expr::Ident(name) => self.lookup_type_val(name),

            Expr::Call(func, args) => {
                if let Expr::Ident(name) = func.as_ref() {
                    if let Some(TypeVal::Fn(_, ret)) = self.lookup_type_val(name) {
                        for arg in args.iter_mut() {
                            self.type_of_expr(arg);
                        }
                        ret.map(|r| *r)
                    } else {
                        self.errors
                            .push(format!("'{}' is not a function", name));
                        None
                    }
                } else {
                    None
                }
            }

            Expr::Binary(op, left, right) => {
                let lt = self.type_of_expr(left);
                let rt = self.type_of_expr(right);

                match op {
                    BinOp::Add
                    | BinOp::Sub
                    | BinOp::Mul
                    | BinOp::Div
                    | BinOp::Mod
                    | BinOp::BitAnd
                    | BinOp::BitOr
                    | BinOp::BitXor
                    | BinOp::Shl
                    | BinOp::Shr => lt.or(rt),

                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                        Some(TypeVal::Bool)
                    }

                    BinOp::And | BinOp::Or => Some(TypeVal::Bool),
                }
            }

            Expr::Unary(_, inner) => self.type_of_expr(inner),

            Expr::Block(block) => {
                let mut last_type = None;
                for stmt in block.stmts.iter_mut() {
                    last_type = self.analyze_stmt(stmt);
                }
                last_type
            }

            Expr::If(cond, then_block, else_branch) => {
                self.type_of_expr(cond);
                let then_t = self.analyze_block(then_block);
                let else_t = else_branch
                    .as_mut()
                    .and_then(|e| self.type_of_expr(e));
                then_t.or(else_t)
            }

            Expr::While(cond, body) => {
                self.type_of_expr(cond);
                self.analyze_block(body);
                None
            }

            Expr::For(iterable, item, index, body) => {
                let iter_type = self.type_of_expr(iterable);
                self.push_scope();

                if let Some(ref it) = iter_type {
                    // Infer element type from slice/array
                    let elem_type = match it {
                        TypeVal::Slice(elem) => Some(elem.as_ref().clone()),
                        TypeVal::Array(_, elem) => Some(elem.as_ref().clone()),
                        _ => None,
                    };
                    if let Some(et) = elem_type {
                        if let Some(item_name) = item {
                            self.add_symbol(
                                item_name.clone(),
                                SymbolKind::Variable { mutable: false },
                                et,
                            );
                        }
                    } else {
                        // If we can't determine the element type, use a generic int
                        if let Some(item_name) = item {
                            self.add_symbol(
                                item_name.clone(),
                                SymbolKind::Variable { mutable: false },
                                TypeVal::Int(IntSize::I32),
                            );
                        }
                    }
                }

                if let Some(index_name) = index {
                    self.add_symbol(
                        index_name.clone(),
                        SymbolKind::Variable { mutable: false },
                        TypeVal::Int(IntSize::Usize),
                    );
                }

                self.analyze_block(body);
                self.pop_scope();
                None
            }

            Expr::Switch(expr, arms, else_arm) => {
                self.type_of_expr(expr);
                let mut result_type = None;
                for arm in arms {
                    for pat in &mut arm.patterns {
                        match pat {
                            SwitchPattern::Expr(e) => {
                                self.type_of_expr(e);
                            }
                            SwitchPattern::Range(start, end) => {
                                self.type_of_expr(start);
                                self.type_of_expr(end);
                            }
                            SwitchPattern::Else => {}
                        }
                    }
                    result_type = self.analyze_block(&mut arm.body).or(result_type);
                }
                if let Some(block) = else_arm {
                    result_type = self.analyze_block(block).or(result_type);
                }
                result_type
            }

            Expr::Assign(lhs, rhs) => {
                self.type_of_expr(lhs);
                let rt = self.type_of_expr(rhs);
                rt
            }

            Expr::Index(arr, index) => {
                let arr_type = self.type_of_expr(arr);
                self.type_of_expr(index);
                match arr_type {
                    Some(TypeVal::Slice(elem)) => Some(*elem),
                    Some(TypeVal::Array(_, elem)) => Some(*elem),
                    Some(TypeVal::Ptr(inner)) => Some(*inner),
                    _ => None,
                }
            }

            Expr::Field(obj, field_name) => {
                let obj_type = self.type_of_expr(obj);
                match obj_type {
                    Some(TypeVal::Struct(fields)) => {
                        fields.iter().find(|(n, _)| n == field_name).map(|(_, t)| t.clone())
                    }
                    _ => None,
                }
            }

            Expr::Slice(arr, start, end) => {
                let arr_type = self.type_of_expr(arr);
                if let Some(s) = start {
                    self.type_of_expr(s);
                }
                if let Some(e) = end {
                    self.type_of_expr(e);
                }
                match arr_type {
                    Some(TypeVal::Slice(elem)) => Some(TypeVal::Slice(elem)),
                    Some(TypeVal::Array(_, elem)) => Some(TypeVal::Slice(elem)),
                    _ => None,
                }
            }

            Expr::Try(inner) => {
                // try unwraps an error union
                let inner_type = self.type_of_expr(inner);
                match inner_type {
                    Some(TypeVal::ErrorUnion(ok_type)) => Some(*ok_type),
                    _ => inner_type,
                }
            }

            Expr::Catch(inner, _var, block) => {
                let inner_type = self.type_of_expr(inner);
                let _block_type = self.analyze_block(block);
                match inner_type {
                    Some(TypeVal::ErrorUnion(ok_type)) => Some(*ok_type),
                    t => t,
                }
            }

            Expr::StructInit(name, fields) => {
                for (_, val) in fields.iter_mut() {
                    self.type_of_expr(val);
                }
                // Look up struct type
                Some(TypeVal::Named(name.clone()))
            }

            Expr::ArrayLit(items) => {
                let mut elem_type = None;
                for item in items.iter_mut() {
                    elem_type = self.type_of_expr(item).or(elem_type);
                }
                elem_type.map(|e| TypeVal::Slice(Box::new(e)))
            }

            Expr::Comptime(block) => {
                self.analyze_block(block);
                None
            }

            Expr::When(cond, then_block, else_block) => {
                self.type_of_expr(cond);
                let then_t = self.analyze_block(then_block);
                let else_t = else_block.as_mut().and_then(|b| self.analyze_block(b));
                then_t.or(else_t)
            }
        }
    }

    // === Type conversion ===

    pub fn type_from_ast(&self, type_: &Type) -> TypeVal {
        match type_ {
            Type::Named(name) => self.resolve_named_type(name),
            Type::Ptr(inner) => TypeVal::Ptr(Box::new(self.type_from_ast(inner))),
            Type::ConstPtr(inner) => TypeVal::ConstPtr(Box::new(self.type_from_ast(inner))),
            Type::NullablePtr(inner) => {
                TypeVal::NullablePtr(Box::new(self.type_from_ast(inner)))
            }
            Type::ManyPtr(inner) => TypeVal::ManyPtr(Box::new(self.type_from_ast(inner))),
            Type::Slice(inner) => TypeVal::Slice(Box::new(self.type_from_ast(inner))),
            Type::Array(n, inner) => TypeVal::Array(*n, Box::new(self.type_from_ast(inner))),
            Type::Optional(inner) => TypeVal::Optional(Box::new(self.type_from_ast(inner))),
            Type::ErrorUnion(inner) => TypeVal::ErrorUnion(Box::new(self.type_from_ast(inner))),
            Type::Fn(params, ret) => TypeVal::Fn(
                params.iter().map(|p| self.type_from_ast(p)).collect(),
                ret.as_ref().map(|r| Box::new(self.type_from_ast(r))),
            ),
            Type::Inferred => TypeVal::Named("Inferred".into()),
        }
    }

    pub fn resolve_named_type(&self, name: &str) -> TypeVal {
        match name {
            "void" => TypeVal::Void,
            "bool" => TypeVal::Bool,
            "i8" => TypeVal::Int(IntSize::I8),
            "i16" => TypeVal::Int(IntSize::I16),
            "i32" => TypeVal::Int(IntSize::I32),
            "i64" => TypeVal::Int(IntSize::I64),
            "u8" => TypeVal::Int(IntSize::U8),
            "u16" => TypeVal::Int(IntSize::U16),
            "u32" => TypeVal::Int(IntSize::U32),
            "u64" => TypeVal::Int(IntSize::U64),
            "isize" => TypeVal::Int(IntSize::Isize),
            "usize" => TypeVal::Int(IntSize::Usize),
            "f32" => TypeVal::Float(FloatSize::F32),
            "f64" => TypeVal::Float(FloatSize::F64),
            "string" => TypeVal::Slice(Box::new(TypeVal::Int(IntSize::U8))),
            _ => TypeVal::Named(name.to_string()),
        }
    }
}

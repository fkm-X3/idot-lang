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
    // Generic function templates: name -> (generic_params, params, return_type, body)
    generic_fns: Vec<FnDecl>,
    // Generic struct templates
    generic_structs: Vec<StructDecl>,
    // Monomorphization counter for unique naming
    mono_counter: usize,
    // Monomorphized function declarations to insert into the program
    pub monomorphized_fns: Vec<FnDecl>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = SemanticAnalyzer {
            scopes: vec![],
            errors: vec![],
            generic_fns: vec![],
            generic_structs: vec![],
            mono_counter: 0,
            monomorphized_fns: vec![],
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

        // Phase 2: resolve function bodies (skip generic templates)
        for decl in program.iter_mut() {
            if let Decl::Fn(f) = decl {
                if f.generic_params.is_empty() {
                    self.analyze_fn_decl(f);
                }
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
                    if !f.generic_params.is_empty() {
                        // Store generic function template for later monomorphization
                        self.generic_fns.push(f.clone());
                        continue;
                    }
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
                    if !s.generic_params.is_empty() {
                        // Store generic struct template for later monomorphization
                        self.generic_structs.push(s.clone());
                        continue;
                    }
                    let mut fields: Vec<(String, TypeVal)> = Vec::new();
                    for f in &s.fields {
                        let ft = self.type_from_ast(&f.type_);
                        if f.using_ {
                            if let TypeVal::Struct(ref inner_fields) = ft {
                                for (iname, itype) in inner_fields {
                                    fields.push((iname.clone(), itype.clone()));
                                }
                            } else if let TypeVal::Named(ref name) = ft {
                                if let Some(resolved) = self.lookup_type_val(name) {
                                    if let TypeVal::Struct(ref inner_fields) = resolved {
                                        for (iname, itype) in inner_fields {
                                            fields.push((iname.clone(), itype.clone()));
                                        }
                                    } else {
                                        fields.push((f.name.clone(), ft.clone()));
                                    }
                                } else {
                                    fields.push((f.name.clone(), ft.clone()));
                                }
                            } else {
                                fields.push((f.name.clone(), ft.clone()));
                            }
                        } else {
                            fields.push((f.name.clone(), ft));
                        }
                    }
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
                Decl::Import(_) | Decl::Union(_) => {
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
            Decl::Let(v) => {
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

            Expr::Ident(name) => self.lookup_type_val(name),

            Expr::Call(func, args) => {
                if let Expr::Ident(name) = func.as_mut() {
                    // Type-check arguments first
                    let arg_types: Vec<Option<TypeVal>> = args.iter_mut()
                        .map(|a| self.type_of_expr(a))
                        .collect();

                    if let Some(TypeVal::Fn(_, ret)) = self.lookup_type_val(name) {
                        ret.map(|r| *r)
                    } else if let Some(mangled) = self.try_monomorphize_fn(name, &arg_types) {
                        // Rewrite call site to use the monomorphized function name
                        *name = mangled;
                        // Look up the return type from the registered symbol
                        self.lookup_type_val(name).and_then(|tv| {
                            if let TypeVal::Fn(_, ret) = tv { ret.map(|r| *r) } else { None }
                        })
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

            Expr::Match(expr, arms, wildcard_arm) => {
                self.type_of_expr(expr);
                let mut result_type = None;
                for arm in arms {
                    for pat in &mut arm.patterns {
                        match pat {
                            MatchPattern::Expr(e) => {
                                self.type_of_expr(e);
                            }
                            MatchPattern::Range(start, end) => {
                                self.type_of_expr(start);
                                self.type_of_expr(end);
                            }
                            MatchPattern::Wildcard => {}
                        }
                    }
                    result_type = self.analyze_block(&mut arm.body).or(result_type);
                }
                if let Some(block) = wildcard_arm {
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

            Expr::StructInit(name, fields) => {
                for (_, val) in fields.iter_mut() {
                    self.type_of_expr(val);
                }
                // Look up struct type
                Some(TypeVal::Named(name.clone()))
            }

            Expr::Deref(inner) => {
                // x.*  — get the pointee type of a pointer
                let inner_t = self.type_of_expr(inner);
                match inner_t {
                    Some(TypeVal::Ptr(t)) => Some(*t),
                    Some(TypeVal::ConstPtr(t)) => Some(*t),
                    Some(TypeVal::NullablePtr(t)) => Some(*t),
                    Some(TypeVal::ManyPtr(t)) => Some(*t),
                    _ => None,
                }
            }

            Expr::Try(inner) => {
                let inner_t = self.type_of_expr(inner);
                match inner_t {
                    Some(TypeVal::ErrorUnion(t)) => Some(*t),
                    _ => {
                        self.errors.push("'try' expression requires an error union type".into());
                        None
                    }
                }
            }

            Expr::Catch(lhs, rhs) => {
                let lhs_t = self.type_of_expr(lhs);
                let rhs_t = self.type_of_expr(rhs);
                // The catch result is the inner type of the error union
                match lhs_t {
                    Some(TypeVal::ErrorUnion(t)) => rhs_t.or(Some(*t)),
                    _ => {
                        self.errors.push("'catch' requires an error union expression".into());
                        rhs_t
                    }
                }
            }

            Expr::OrElse(lhs, rhs) => {
                let lhs_t = self.type_of_expr(lhs);
                self.type_of_expr(rhs);
                // The result is the inner type of the optional
                match lhs_t {
                    Some(TypeVal::Optional(t)) => Some(*t),
                    Some(TypeVal::NullablePtr(t)) => Some(*t),
                    _ => lhs_t,
                }
            }

            Expr::Comptime(inner) => {
                // Evaluate the inner expression at compile time
                // For now, just type-check it
                self.type_of_expr(inner)
            }

            Expr::When(cond, then_block, _else_branch) => {
                // when is compile-time branching; only the then-block is semantically analyzed
                // (the condition is assumed true — future comptime evaluation will select the branch)
                self.type_of_expr(cond);
                self.analyze_block(then_block)
            }

            Expr::ArrayLit(items) => {
                let mut elem_type = None;
                for item in items.iter_mut() {
                    elem_type = self.type_of_expr(item).or(elem_type);
                }
                elem_type.map(|e| TypeVal::Slice(Box::new(e)))
            }
        }
    }

    // === Generics: Monomorphization ===

    fn try_monomorphize_fn(&mut self, name: &str, arg_types: &[Option<TypeVal>]) -> Option<String> {
        // Find matching generic function template
        let template = self.generic_fns.iter().find(|f| f.name == name)?.clone();

        // Build type substitution map from generic params to concrete types
        let mut type_map: std::collections::HashMap<String, TypeVal> = std::collections::HashMap::new();

        // For each generic param, infer its concrete type from argument types
        for (i, gparam) in template.generic_params.iter().enumerate() {
            if let Some(Some(arg_t)) = arg_types.get(i) {
                // If the param has a constraint (e.g., `type`), verify it
                // For now, just map it
                type_map.insert(gparam.name.clone(), arg_t.clone());
            }
        }

        // Create a mangled name for the monomorphized version
        let mangled = format!("{}__mono{}", name, self.mono_counter);
        self.mono_counter += 1;

        // Clone the template — keep the original AST types (e.g., Type::Named("T"))
        // so analyze_fn_decl can set resolved types from them.
        let mut mono_fn = template.clone();
        mono_fn.name = mangled;
        mono_fn.generic_params = Vec::new(); // resolved

        // Analyze the monomorphized function body.
        // This resolves types via type_from_ast, producing TypeVal::Named("T") for
        // generic params — which we override below with concrete types from the type_map.
        self.analyze_fn_decl(&mut mono_fn);

        // Override resolved types with concrete types from the type_map
        for param in mono_fn.params.iter_mut() {
            if let Some(ref tv) = param.resolved_type {
                param.resolved_type = Some(self.substitute_type_val(tv, &type_map));
            }
        }
        mono_fn.resolved_ret_type = mono_fn.resolved_ret_type.as_ref()
            .map(|tv| self.substitute_type_val(tv, &type_map));

        // Store the monomorphized function for codegen
        let mangled_name = mono_fn.name.clone();
        self.monomorphized_fns.push(mono_fn);

        Some(mangled_name)
    }

    fn substitute_type(&self, type_: &Type, type_map: &std::collections::HashMap<String, TypeVal>) -> Type {
        match type_ {
            Type::Named(name) => {
                if type_map.contains_key(name) {
                    Type::Named(format!("__mono_{}", name))
                } else {
                    type_.clone()
                }
            }
            Type::Ptr(inner) => Type::Ptr(Box::new(self.substitute_type(inner, type_map))),
            Type::ConstPtr(inner) => Type::ConstPtr(Box::new(self.substitute_type(inner, type_map))),
            Type::NullablePtr(inner) => Type::NullablePtr(Box::new(self.substitute_type(inner, type_map))),
            Type::ManyPtr(inner) => Type::ManyPtr(Box::new(self.substitute_type(inner, type_map))),
            Type::Slice(inner) => Type::Slice(Box::new(self.substitute_type(inner, type_map))),
            Type::Array(n, inner) => Type::Array(*n, Box::new(self.substitute_type(inner, type_map))),
            Type::Optional(inner) => Type::Optional(Box::new(self.substitute_type(inner, type_map))),
            Type::ErrorUnion(inner) => Type::ErrorUnion(Box::new(self.substitute_type(inner, type_map))),
            Type::Fn(params, ret) => Type::Fn(
                params.iter().map(|p| self.substitute_type(p, type_map)).collect(),
                ret.as_ref().map(|r| Box::new(self.substitute_type(r, type_map))),
            ),
            Type::Inferred => Type::Inferred,
        }
    }

    fn substitute_type_val(&self, tv: &TypeVal, type_map: &std::collections::HashMap<String, TypeVal>) -> TypeVal {
        match tv {
            TypeVal::Named(name) => {
                type_map.get(name).cloned().unwrap_or(tv.clone())
            }
            TypeVal::Ptr(inner) => TypeVal::Ptr(Box::new(self.substitute_type_val(inner, type_map))),
            TypeVal::ConstPtr(inner) => TypeVal::ConstPtr(Box::new(self.substitute_type_val(inner, type_map))),
            TypeVal::NullablePtr(inner) => TypeVal::NullablePtr(Box::new(self.substitute_type_val(inner, type_map))),
            TypeVal::ManyPtr(inner) => TypeVal::ManyPtr(Box::new(self.substitute_type_val(inner, type_map))),
            TypeVal::Slice(inner) => TypeVal::Slice(Box::new(self.substitute_type_val(inner, type_map))),
            TypeVal::Array(n, inner) => TypeVal::Array(*n, Box::new(self.substitute_type_val(inner, type_map))),
            TypeVal::Optional(inner) => TypeVal::Optional(Box::new(self.substitute_type_val(inner, type_map))),
            TypeVal::ErrorUnion(inner) => TypeVal::ErrorUnion(Box::new(self.substitute_type_val(inner, type_map))),
            TypeVal::Struct(fields) => TypeVal::Struct(
                fields.iter().map(|(n, ft)| (n.clone(), self.substitute_type_val(ft, type_map))).collect()
            ),
            TypeVal::Fn(params, ret) => TypeVal::Fn(
                params.iter().map(|p| self.substitute_type_val(p, type_map)).collect(),
                ret.as_ref().map(|r| Box::new(self.substitute_type_val(r, type_map))),
            ),
            other => other.clone(),
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

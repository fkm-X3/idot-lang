use crate::ast::*;

pub struct CBackend {
    output: String,
    indent: usize,
    fn_decls: Vec<String>,
    emitted_types: std::collections::HashSet<String>,
    for_counter: usize,
    try_counter: usize,
    var_types: std::collections::HashMap<String, TypeVal>,
    defer_stack: Vec<Vec<String>>,   // per-scope deferred expressions
    in_function: bool,
    struct_fields: std::collections::HashMap<String, Vec<(String, Type)>>,
}

impl CBackend {
    pub fn new() -> Self {
        CBackend {
            output: String::new(),
            indent: 0,
            fn_decls: Vec::new(),
            emitted_types: std::collections::HashSet::new(),
            for_counter: 0,
            try_counter: 0,
            var_types: std::collections::HashMap::new(),
            defer_stack: Vec::new(),
            in_function: false,
            struct_fields: std::collections::HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &[Decl]) -> String {
        self.output.clear();
        self.emit_header();

        // First pass: collect function declarations
        for decl in program {
            let mut be = CBackend::new();
            if let Decl::Fn(f) = decl {
                if f.is_extern {
                    be.emit_extern_fn(f);
                } else {
                    be.emit_fn_sig(f);
                    be.emit_line(";");
                }
                self.fn_decls.push(be.output);
            }
        }

        // Emit type definitions (struct, enum, union) BEFORE function declarations
        for decl in program {
            self.emit_type_def(decl);
        }

        // Emit function declarations
        for d in &self.fn_decls {
            self.output.push_str(d);
            self.output.push('\n');
        }

        // Second pass: emit full function definitions and globals
        for decl in program {
            match decl {
                Decl::Fn(f) if !f.is_extern => {
                    self.emit_fn_def(f);
                }
                Decl::Let(v) => {
                    self.emit_global_var(v);
                }
                Decl::Const(c) => {
                    self.emit_global_const(c);
                }
                _ => {}
            }
        }

        self.output.clone()
    }

    fn emit_type_def(&mut self, decl: &Decl) {
        match decl {
            Decl::Struct(s) if !s.name.is_empty() => {
                if self.emitted_types.contains(&s.name) {
                    return;
                }
                self.emitted_types.insert(s.name.clone());
                self.struct_fields.insert(s.name.clone(), s.fields.iter().map(|f| (f.name.clone(), f.type_.clone())).collect());
                self.output.push_str("typedef struct ");
                self.output.push_str(&s.name);
                self.output.push_str(" { ");
                let mut field_idx = 0;
                for field in &s.fields {
                    if field.using_ {
                        // Expand using fields: inline the referenced struct's fields
                        if let Type::Named(ref _type_name) = field.type_ {
                            // We don't have the full struct registry here, so just emit
                            // the using field as a regular nested struct member.
                            // The semantic analyzer handles field promotion.
                            if field_idx > 0 { self.output.push_str("; "); }
                            self.emit_type(&field.type_);
                            self.output.push(' ');
                            self.output.push_str(&field.name);
                            field_idx += 1;
                        } else {
                            if field_idx > 0 { self.output.push_str("; "); }
                            self.emit_type(&field.type_);
                            self.output.push(' ');
                            self.output.push_str(&field.name);
                            field_idx += 1;
                        }
                    } else {
                        if field_idx > 0 { self.output.push_str("; "); }
                        self.emit_type(&field.type_);
                        self.output.push(' ');
                        self.output.push_str(&field.name);
                        field_idx += 1;
                    }
                }
                self.output.push_str("; } ");
                self.output.push_str(&s.name);
                self.emit_line(";");
                self.emit_line("");
            }
            Decl::Enum(e) if !e.name.is_empty() => {
                if self.emitted_types.contains(&e.name) {
                    return;
                }
                self.emitted_types.insert(e.name.clone());
                self.output.push_str("typedef enum ");
                self.output.push_str(&e.name);
                self.output.push_str(" { ");
                for (i, v) in e.variants.iter().enumerate() {
                    if i > 0 { self.output.push_str(", "); }
                    self.output.push_str(&v.name);
                    if let Some(ref val) = v.value {
                        self.output.push_str(" = ");
                        self.emit_expr(val);
                    }
                }
                self.output.push_str(" } ");
                self.output.push_str(&e.name);
                self.emit_line(";");
                self.emit_line("");
            }
            _ => {}
        }
    }

    fn emit_header(&mut self) {
        self.emit_line("#include <stddef.h>");
        self.emit_line("#include <stdint.h>");
        self.emit_line("#include <stdbool.h>");
        self.emit_line("#include <string.h>");
        self.emit_line("#include <stdlib.h>");
        self.emit_line("#include <math.h>");
        // Include platform-specific headers for POSIX I/O and O_* flags
        self.emit_line("#if defined(_WIN32) || defined(_WIN64)");
        self.emit_line("#include <io.h>"); // _open, _write, _close on Windows (mingw/MSVC)
        self.emit_line("#else");
        self.emit_line("#include <unistd.h>"); // for open, write, close, read, lseek
        self.emit_line("#endif");
        self.emit_line("#include <fcntl.h>"); // for O_* flags
        self.emit_line("#include <stdio.h>"); // for SEEK_* constants
        self.emit_line("typedef struct { uint8_t* ptr; size_t len; } IdotString;");
        self.emit_line("static int __ido_argc;");
        self.emit_line("static uint8_t** __ido_argv;");
        self.emit_line("int __ido_get_argc(void) { return __ido_argc; }");
        self.emit_line("uint8_t* __ido_get_argv(int i) { return __ido_argv[i]; }");
        self.emit_line("");
    }

    // === Type emission ===

    fn emit_type(&mut self, type_: &Type) {
        match type_ {
            Type::Named(name) => self.emit_type_name(name),
            Type::Ptr(inner) => {
                self.emit_type(inner);
                self.output.push('*');
            }
            Type::ConstPtr(inner) => {
                self.output.push_str("const ");
                self.emit_type(inner);
                self.output.push('*');
            }
            Type::NullablePtr(inner) => {
                self.emit_type(inner);
                self.output.push('*');
            }
            Type::ManyPtr(inner) => {
                self.emit_type(inner);
                self.output.push('*');
            }
            Type::Slice(inner) => {
                // Use IdotString typedef for []u8 (the string type)
                if let Type::Named(name) = inner.as_ref() {
                    if name == "u8" {
                        self.output.push_str("IdotString");
                    } else {
                        self.output.push_str("struct { ");
                        self.emit_type(inner);
                        self.output.push_str("* ptr; size_t len; }");
                    }
                } else {
                    self.output.push_str("struct { ");
                    self.emit_type(inner);
                    self.output.push_str("* ptr; size_t len; }");
                }
            }
            Type::Array(size, inner) => {
                self.emit_type(inner);
                self.output.push(' ');
                // We handle array sizing at the variable level
                self.output.push_str(&format!("[{}]", size));
            }
            Type::Optional(inner) => {
                // Represent as struct { bool has; T val; }
                self.output.push_str("struct { bool has; ");
                self.emit_type(inner);
                self.output.push_str(" val; }");
            }
            Type::ErrorUnion(inner) => {
                // Represent as struct { uintptr_t err; union {T val;} data; int _tag; }
                self.output.push_str("struct { uintptr_t err; union {");
                self.emit_type(inner);
                self.output.push_str(" val; } data; }");
            }
            Type::Fn(params, ret) => {
                if let Some(ret) = ret {
                    self.emit_type(ret);
                } else {
                    self.output.push_str("void");
                }
                self.output.push_str(" (*)(");
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { self.output.push_str(", "); }
                    self.emit_type(p);
                }
                self.output.push(')');
            }
            Type::Inferred => {
                self.output.push_str("int"); // fallback
            }
        }
    }

    fn emit_type_name(&mut self, name: &str) {
        match name {
            "i8" => self.output.push_str("int8_t"),
            "i16" => self.output.push_str("int16_t"),
            "i32" => self.output.push_str("int32_t"),
            "i64" => self.output.push_str("int64_t"),
            "u8" => self.output.push_str("uint8_t"),
            "u16" => self.output.push_str("uint16_t"),
            "u32" => self.output.push_str("uint32_t"),
            "u64" => self.output.push_str("uint64_t"),
            "isize" => self.output.push_str("intptr_t"),
            "usize" => self.output.push_str("size_t"),
            "f32" => self.output.push_str("float"),
            "f64" => self.output.push_str("double"),
            "bool" => self.output.push_str("bool"),
            "void" => self.output.push_str("void"),
            "string" => self.output.push_str("IdotString"),
            _ => self.output.push_str(name), // user-defined types
        }
    }

    fn emit_type_val(&mut self, tv: &TypeVal) {
        match tv {
            TypeVal::Void => self.output.push_str("void"),
            TypeVal::Bool => self.output.push_str("bool"),
            TypeVal::Int(size) => match size {
                IntSize::I8 => self.output.push_str("int8_t"),
                IntSize::I16 => self.output.push_str("int16_t"),
                IntSize::I32 => self.output.push_str("int32_t"),
                IntSize::I64 => self.output.push_str("int64_t"),
                IntSize::U8 => self.output.push_str("uint8_t"),
                IntSize::U16 => self.output.push_str("uint16_t"),
                IntSize::U32 => self.output.push_str("uint32_t"),
                IntSize::U64 => self.output.push_str("uint64_t"),
                IntSize::Isize => self.output.push_str("intptr_t"),
                IntSize::Usize => self.output.push_str("size_t"),
            },
            TypeVal::Float(size) => match size {
                FloatSize::F32 => self.output.push_str("float"),
                FloatSize::F64 => self.output.push_str("double"),
            },
            TypeVal::Ptr(inner) => {
                self.emit_type_val(inner);
                self.output.push('*');
            }
            TypeVal::ConstPtr(inner) => {
                self.output.push_str("const ");
                self.emit_type_val(inner);
                self.output.push('*');
            }
            TypeVal::NullablePtr(inner) => {
                self.emit_type_val(inner);
                self.output.push('*');
            }
            TypeVal::ManyPtr(inner) => {
                self.emit_type_val(inner);
                self.output.push('*');
            }
            TypeVal::Slice(inner) => {
                // Use IdotString typedef for []u8 (the string type)
                if matches!(inner.as_ref(), TypeVal::Int(IntSize::U8)) {
                    self.output.push_str("IdotString");
                } else {
                    self.output.push_str("struct { ");
                    self.emit_type_val(inner);
                    self.output.push_str("* ptr; size_t len; }");
                }
            }
            TypeVal::Array(size, inner) => {
                self.emit_type_val(inner);
                self.output.push_str(&format!(" [{}]", size));
            }
            TypeVal::Optional(inner) => {
                self.output.push_str("struct { bool has; ");
                self.emit_type_val(inner);
                self.output.push_str(" val; }");
            }
            TypeVal::ErrorUnion(inner) => {
                self.output.push_str("struct { uintptr_t err; union {");
                self.emit_type_val(inner);
                self.output.push_str(" val; } data; }");
            }
            TypeVal::Fn(params, ret) => {
                if let Some(ret) = ret {
                    self.emit_type_val(ret);
                } else {
                    self.output.push_str("void");
                }
                self.output.push_str(" (*)(");
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { self.output.push_str(", "); }
                    self.emit_type_val(p);
                }
                self.output.push(')');
            }
            TypeVal::Struct(fields) => {
                self.output.push_str("struct { ");
                for (i, (name, ft)) in fields.iter().enumerate() {
                    if i > 0 { self.output.push_str("; "); }
                    self.emit_type_val(ft);
                    self.output.push(' ');
                    self.output.push_str(name);
                }
                self.output.push_str("; }");
            }
            TypeVal::Named(name) => {
                self.emit_type_name(name);
            }
        }
    }

    // === Declarations ===

    fn emit_extern_fn(&mut self, f: &FnDecl) {
        // Skip emission for standard library functions declared by the
        // included system headers (<string.h>, <stdlib.h>, <math.h>,
        // <unistd.h>/<io.h>). Their C signatures differ from what Idot
        // generates (e.g. uint8_t* vs void*/const char*), causing
        // conflicting-declaration errors on Linux.
        const STD_FUNCS: &[&str] = &[
            // POSIX I/O (<unistd.h> on Linux, <io.h> on Windows)
            "write", "read", "open", "close", "lseek",
            // <string.h>
            "memset", "memcpy", "memmove", "strlen", "strcmp", "strncmp",
            // <stdlib.h>
            "malloc", "calloc", "realloc", "free", "exit", "system", "getenv",
            // <math.h>
            "sqrt", "sin", "cos", "tan", "pow", "floor", "ceil", "fabs", "fmod",
        ];
        if STD_FUNCS.contains(&f.name.as_str()) {
            return;
        }
        self.output.push_str("extern ");
        self.emit_fn_sig(f);
        self.emit_line(";");
    }

    fn emit_fn_sig(&mut self, f: &FnDecl) {
        if f.name == "main" {
            self.output.push_str("int main(int argc, char** argv)");
            return;
        }
        if let Some(ret) = &f.resolved_ret_type {
            self.emit_type_val(ret);
        } else if let Some(ret) = &f.return_type {
            self.emit_type(ret);
        } else {
            self.output.push_str("void");
        }
        self.output.push(' ');
        self.output.push_str(&f.name);
        self.output.push('(');
        for (i, p) in f.params.iter().enumerate() {
            if i > 0 { self.output.push_str(", "); }
            self.emit_param_type(&p.type_, &p.resolved_type);
            self.output.push(' ');
            self.output.push_str(&p.name);
        }
        self.output.push(')');
    }

    fn emit_param_type(&mut self, type_: &Type, resolved: &Option<TypeVal>) {
        if let Some(tv) = resolved {
            self.emit_type_val(tv);
        } else {
            self.emit_type(type_);
        }
    }

    fn emit_fn_def(&mut self, f: &FnDecl) {
        self.emit_fn_sig(f);
        self.emit_line(" {");
        self.indent += 1;
        self.in_function = true;
        self.defer_stack.push(Vec::new());
        if f.name == "main" {
            self.emit_line("__ido_argc = argc;");
            self.emit_line("__ido_argv = (uint8_t**)argv;");
        }
        for p in &f.params {
            if let Some(tv) = &p.resolved_type {
                self.var_types.insert(p.name.clone(), tv.clone());
            }
        }
        if !f.body.stmts.is_empty() {
            for stmt in &f.body.stmts {
                self.emit_stmt(stmt);
            }
        }
        // If no explicit return, emit defers before implicit return
        let has_explicit_return = f.body.stmts.iter().any(|s| matches!(s, Stmt::Return(_)));
        if !has_explicit_return {
            self.emit_pending_defers();
        }
        self.defer_stack.pop();
        // If void return and no return statement, add implicit return
        if f.return_type.is_none() || matches!(&f.return_type, Some(Type::Named(n)) if n == "void") {
            let has_return = f.body.stmts.iter().any(|s| matches!(s, Stmt::Return(_)));
            if !has_return {
                self.emit_line("return;");
            }
        }
        self.indent -= 1;
        self.in_function = false;
        self.emit_line("}");
        self.emit_line("");
    }

    fn emit_pending_defers(&mut self) {
        let mut pending: Vec<String> = self.defer_stack.last()
            .cloned()
            .unwrap_or_default();
        pending.reverse();
        for expr_str in &pending {
            self.emit_indent();
            self.output.push_str(expr_str);
            self.emit_line(";");
        }
    }

    fn emit_global_var(&mut self, v: &VarDecl) {
        if let Some(tv) = &v.resolved_type {
            self.var_types.insert(v.name.clone(), tv.clone());
        }
        self.output.push_str(if v.mutable { "" } else { "const " });
        if let Some(tv) = &v.resolved_type {
            self.emit_type_val(tv);
        } else if let Some(type_) = &v.type_ {
            self.emit_type(type_);
        } else {
            self.output.push_str("int"); // inferred fallback
        }
        self.output.push(' ');
        self.output.push_str(&v.name);
        if let Some(init) = &v.init {
            self.output.push_str(" = ");
            self.emit_expr(init);
        }
        self.emit_line(";");
    }

    fn emit_global_const(&mut self, c: &ConstDecl) {
        // Skip constants already defined as macros by the included system
        // headers (<fcntl.h>, <stdio.h>, <unistd.h>).
        const SYS_CONSTS: &[&str] = &[
            "STDIN_FILENO", "STDOUT_FILENO", "STDERR_FILENO",
            "O_RDONLY", "O_WRONLY", "O_RDWR",
            "O_CREAT", "O_TRUNC", "O_APPEND",
            "SEEK_SET", "SEEK_CUR", "SEEK_END",
        ];
        if SYS_CONSTS.contains(&c.name.as_str()) {
            return;
        }
        if let Some(tv) = &c.resolved_type {
            self.var_types.insert(c.name.clone(), tv.clone());
        }
        self.output.push_str("const ");
        if let Some(tv) = &c.resolved_type {
            self.emit_type_val(tv);
        } else if let Some(type_) = &c.type_ {
            self.emit_type(type_);
        } else {
            self.output.push_str("int");
        }
        self.output.push(' ');
        self.output.push_str(&c.name);
        self.output.push_str(" = ");
        self.emit_expr(&c.init);
        self.emit_line(";");
    }

    // === Statements ===

    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Decl(decl) => match decl {
                Decl::Fn(f) => self.emit_fn_def(f),
                Decl::Let(v) => {
                    if let Some(tv) = &v.resolved_type {
                        self.var_types.insert(v.name.clone(), tv.clone());
                    }
                    self.emit_indent();
                    if let Some(tv) = &v.resolved_type {
                        self.emit_type_val(tv);
                    } else if let Some(type_) = &v.type_ {
                        self.emit_type(type_);
                    } else {
                        self.output.push_str("int");
                    }
                    self.output.push(' ');
                    self.output.push_str(&v.name);
                    if let Some(init) = &v.init {
                        self.output.push_str(" = ");
                        // When initializing a Slice with an ArrayLit, emit array compound literal syntax
                        if let Some(TypeVal::Slice(elem)) = &v.resolved_type {
                            if let Expr::ArrayLit(items) = init {
                                let count = items.len();
                                self.output.push_str("{(");
                                self.emit_type_val(elem);
                                self.output.push_str("[]){");
                                for (i, item) in items.iter().enumerate() {
                                    if i > 0 { self.output.push_str(", "); }
                                    self.emit_expr(item);
                                }
                                self.output.push_str("}, ");
                                self.output.push_str(&count.to_string());
                                self.output.push('}');
                                self.emit_line(";");
                                return;
                            }
                        }
                        self.emit_expr(init);
                    }
                    self.emit_line(";");
                }
                Decl::Const(c) => {
                    if let Some(tv) = &c.resolved_type {
                        self.var_types.insert(c.name.clone(), tv.clone());
                    }
                    self.emit_indent();
                    self.output.push_str("const ");
                    if let Some(tv) = &c.resolved_type {
                        self.emit_type_val(tv);
                    } else if let Some(type_) = &c.type_ {
                        self.emit_type(type_);
                    } else {
                        self.output.push_str("int");
                    }
                    self.output.push(' ');
                    self.output.push_str(&c.name);
                    self.output.push_str(" = ");
                    self.emit_expr(&c.init);
                    self.emit_line(";");
                }
                _ => {}
            },
            Stmt::Expr(expr) => {
                self.emit_indent();
                self.emit_expr(expr);
                self.emit_line(";");
            }
            Stmt::Return(expr) => {
                self.emit_pending_defers();
                self.emit_indent();
                self.output.push_str("return");
                if let Some(e) = expr {
                    self.output.push(' ');
                    self.emit_expr(e);
                }
                self.emit_line(";");
            }
            Stmt::Break => {
                self.emit_indent();
                self.emit_line("break;");
            }
            Stmt::Continue => {
                self.emit_indent();
                self.emit_line("continue;");
            }
            Stmt::Defer(expr) => {
                // Record the deferred expression to emit at scope exit
                let mut be = CBackend::new();
                be.emit_expr(expr);
                if let Some(defers) = self.defer_stack.last_mut() {
                    defers.push(be.output);
                }
            }
            Stmt::Errdefer(expr) => {
                // For now, errdefer acts like defer (simplified)
                let mut be = CBackend::new();
                be.emit_expr(expr);
                if let Some(defers) = self.defer_stack.last_mut() {
                    defers.push(be.output);
                }
            }
        }
    }

    // === Expressions ===

    fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::IntLit(n) => self.output.push_str(&n.to_string()),
            Expr::FloatLit(n) => self.output.push_str(&n.to_string()),
            Expr::StrLit(s) => {
                let mut escaped = String::new();
                for c in s.chars() {
                    match c {
                        '\n' => escaped.push_str("\\n"),
                        '\t' => escaped.push_str("\\t"),
                        '\r' => escaped.push_str("\\r"),
                        '\\' => escaped.push_str("\\\\"),
                        '"' => escaped.push_str("\\\""),
                        '\0' => escaped.push_str("\\0"),
                        c if c.is_ascii() => escaped.push(c),
                        _ => escaped.push(c),
                    }
                }
                self.output.push_str("((IdotString){(uint8_t*)\"");
                self.output.push_str(&escaped);
                self.output.push_str("\", sizeof(\"");
                self.output.push_str(&escaped);
                self.output.push_str("\") - 1})");
            }
            Expr::CharLit(c) => {
                self.output.push('\'');
                match c {
                    '\n' => self.output.push_str("\\n"),
                    '\t' => self.output.push_str("\\t"),
                    '\'' => self.output.push_str("\\'"),
                    '\\' => self.output.push_str("\\\\"),
                    c => self.output.push(*c),
                }
                self.output.push('\'');
            }
            Expr::BoolLit(b) => self.output.push_str(if *b { "true" } else { "false" }),
            Expr::NullLit => self.output.push_str("NULL"),
            Expr::Ident(name) => self.output.push_str(name),
            Expr::Block(block) => {
                self.emit_line(" {");
                self.indent += 1;
                for stmt in &block.stmts {
                    self.emit_stmt(stmt);
                }
                self.indent -= 1;
                self.emit_indent();
                self.output.push('}');
            }
            Expr::If(cond, then_block, else_branch) => {
                // Check if both branches are single-expression blocks (not control-flow) — emit as ternary
                let is_simple_ternary = then_block.stmts.len() == 1
                    && matches!(&then_block.stmts[0], Stmt::Expr(e) if !matches!(e, Expr::If(..) | Expr::For(..) | Expr::While(..) | Expr::Match(..)))
                    && else_branch.as_ref().is_some_and(|e| {
                        matches!(e.as_ref(), Expr::Block(b) if b.stmts.len() == 1
                            && matches!(&b.stmts[0], Stmt::Expr(_)))
                    });
                if is_simple_ternary {
                    self.output.push('(');
                    self.emit_expr(cond);
                    self.output.push_str(" ? (");
                    if let Stmt::Expr(e) = &then_block.stmts[0] {
                        self.emit_expr(e);
                    }
                    self.output.push_str(") : (");
                    if let Some(else_expr) = else_branch.as_ref() {
                        if let Expr::Block(b) = else_expr.as_ref() {
                            if let Stmt::Expr(e) = &b.stmts[0] {
                                self.emit_expr(e);
                            }
                        }
                    }
                    self.output.push_str("))");
                } else {
                    self.output.push_str("if (");
                    self.emit_expr(cond);
                    self.output.push_str(") ");
                    self.emit_block_as_stmt(then_block);
                    if let Some(else_expr) = else_branch {
                        self.output.push_str(" else ");
                        match else_expr.as_ref() {
                            Expr::If(..) => self.emit_expr(else_expr),
                            Expr::Block(b) => self.emit_block_as_stmt(b),
                            _ => {
                                self.output.push_str("{ ");
                                self.emit_expr(else_expr);
                                self.output.push_str("; }");
                            }
                        }
                    }
                }
            }
            Expr::For(iterable, item, index, body) => {
                let idx = self.for_counter;
                self.for_counter += 1;
                // Check if iterable is a variable with a known type
                let iter_type = match iterable.as_ref() {
                    Expr::Ident(name) => self.var_types.get(name),
                    _ => None,
                };

                if iter_type.map_or(false, |t| matches!(t, TypeVal::Slice(_))) {
                    // Slice iteration: use .ptr and .len
                    let idx_str = idx.to_string();
                    self.output.push_str("{ size_t _for_len");
                    self.output.push_str(&idx_str);
                    self.output.push_str(" = ");
                    self.emit_expr(iterable);
                    self.output.push_str(".len; ");
                    self.output.push_str("for (size_t _for_i");
                    self.output.push_str(&idx_str);
                    self.output.push_str(" = 0; _for_i");
                    self.output.push_str(&idx_str);
                    self.output.push_str(" < _for_len");
                    self.output.push_str(&idx_str);
                    self.output.push_str("; _for_i");
                    self.output.push_str(&idx_str);
                    self.output.push_str("++) { ");
                    if let Some(item_name) = item {
                        self.output.push_str("int ");
                        self.output.push_str(item_name);
                        self.output.push_str(" = ");
                        self.emit_expr(iterable);
                        self.output.push_str(".ptr[_for_i");
                        self.output.push_str(&idx_str);
                        self.output.push_str("]; ");
                    }
                    if let Some(idx_name) = index {
                        self.output.push_str("size_t ");
                        self.output.push_str(idx_name);
                        self.output.push_str(" = _for_i");
                        self.output.push_str(&idx_str);
                        self.output.push_str("; ");
                    }
                    for stmt in &body.stmts {
                        self.emit_stmt(stmt);
                    }
                    self.output.push_str(" } }");
                } else if iter_type.map_or(false, |t| matches!(t, TypeVal::Array(_, _))) {
                    // Array iteration: use sizeof/sizeof
                    let idx_str = idx.to_string();
                    self.output.push_str("{ size_t _for_len");
                    self.output.push_str(&idx_str);
                    self.output.push_str(" = sizeof(");
                    self.emit_expr(iterable);
                    self.output.push_str(")/sizeof(");
                    self.emit_expr(iterable);
                    self.output.push_str("[0]); ");
                    self.output.push_str("for (size_t _for_i");
                    self.output.push_str(&idx_str);
                    self.output.push_str(" = 0; _for_i");
                    self.output.push_str(&idx_str);
                    self.output.push_str(" < _for_len");
                    self.output.push_str(&idx_str);
                    self.output.push_str("; _for_i");
                    self.output.push_str(&idx_str);
                    self.output.push_str("++) { ");
                    if let Some(item_name) = item {
                        self.output.push_str("int ");
                        self.output.push_str(item_name);
                        self.output.push_str(" = ");
                        self.emit_expr(iterable);
                        self.output.push_str("[_for_i");
                        self.output.push_str(&idx_str);
                        self.output.push_str("]; ");
                    }
                    if let Some(idx_name) = index {
                        self.output.push_str("size_t ");
                        self.output.push_str(idx_name);
                        self.output.push_str(" = _for_i");
                        self.output.push_str(&idx_str);
                        self.output.push_str("; ");
                    }
                    for stmt in &body.stmts {
                        self.emit_stmt(stmt);
                    }
                    self.output.push_str(" } }");
                } else {
                    // Unknown type: emit placeholder
                    self.output.push_str("{ /* for loop: ");
                    self.emit_expr(iterable);
                    self.output.push_str(" */ ");
                    self.emit_block_as_stmt(body);
                    self.output.push_str(" }");
                }
            }
            Expr::While(cond, body) => {
                self.output.push_str("while (");
                self.emit_expr(cond);
                self.output.push_str(") ");
                self.emit_block_as_stmt(body);
            }
            Expr::Match(expr, arms, wildcard_arm) => {
                let has_range = arms.iter().any(|arm| {
                    arm.patterns.iter().any(|p| matches!(p, MatchPattern::Range(..)))
                });
                if has_range {
                    self.output.push_str("/* switch */ { ");
                    self.output.push_str("int _sw_val = (");
                    self.emit_expr(expr);
                    self.output.push_str("); ");
                    for (ai, arm) in arms.iter().enumerate() {
                        if ai > 0 { self.output.push_str(" else "); }
                        self.output.push_str("if (");
                        for (pi, pat) in arm.patterns.iter().enumerate() {
                            if pi > 0 { self.output.push_str(" || "); }
                            match pat {
                                MatchPattern::Expr(e) => {
                                    self.output.push_str("_sw_val == (");
                                    self.emit_expr(e);
                                    self.output.push(')');
                                }
                                MatchPattern::Range(start, end) => {
                                    self.output.push_str("(_sw_val >= ");
                                    self.emit_expr(start);
                                    self.output.push_str(" && _sw_val <= ");
                                    self.emit_expr(end);
                                    self.output.push(')');
                                }
                                MatchPattern::Wildcard => {}
                            }
                        }
                        self.output.push_str(") ");
                        self.emit_block_as_stmt(&arm.body);
                    }
                    if let Some(eb) = wildcard_arm {
                        self.output.push_str(" else ");
                        self.emit_block_as_stmt(eb);
                    }
                    self.output.push_str(" }");
                } else {
                    self.output.push_str("switch (");
                    self.emit_expr(expr);
                    self.output.push_str(") {");
                    for arm in arms {
                        for pat in &arm.patterns {
                            match pat {
                                MatchPattern::Expr(e) => {
                                    self.output.push_str("case ");
                                    self.emit_expr(e);
                                    self.emit_line(":");
                                }
                                _ => {}
                            }
                        }
                        for stmt in &arm.body.stmts {
                            self.emit_stmt(stmt);
                        }
                    }
                    if let Some(else_block) = wildcard_arm {
                        self.emit_line("default:");
                        for stmt in &else_block.stmts {
                            self.emit_stmt(stmt);
                        }
                    }
                    self.emit_line("}");
                }
            }
            Expr::Unary(op, inner) => {
                match op {
                    UnOp::Neg => { self.output.push('-'); self.emit_expr(inner); }
                    UnOp::Not => { self.output.push('!'); self.emit_expr(inner); }
                    UnOp::Addr => { self.output.push('&'); self.emit_expr(inner); }
                    UnOp::Deref => {
                        self.output.push_str("(*");
                        self.emit_expr(inner);
                        self.output.push(')');
                    }
                }
            }
            Expr::Binary(op, left, right) => {
                self.output.push('(');
                self.emit_expr(left);
                match op {
                    BinOp::Add => self.output.push_str(" + "),
                    BinOp::Sub => self.output.push_str(" - "),
                    BinOp::Mul => self.output.push_str(" * "),
                    BinOp::Div => self.output.push_str(" / "),
                    BinOp::Mod => self.output.push_str(" % "),
                    BinOp::Eq => self.output.push_str(" == "),
                    BinOp::Ne => self.output.push_str(" != "),
                    BinOp::Lt => self.output.push_str(" < "),
                    BinOp::Gt => self.output.push_str(" > "),
                    BinOp::Le => self.output.push_str(" <= "),
                    BinOp::Ge => self.output.push_str(" >= "),
                    BinOp::And => self.output.push_str(" && "),
                    BinOp::Or => self.output.push_str(" || "),
                    BinOp::BitAnd => self.output.push_str(" & "),
                    BinOp::BitOr => self.output.push_str(" | "),
                    BinOp::BitXor => self.output.push_str(" ^ "),
                    BinOp::Shl => self.output.push_str(" << "),
                    BinOp::Shr => self.output.push_str(" >> "),
                }
                self.emit_expr(right);
                self.output.push(')');
            }
            Expr::Assign(lhs, rhs) => {
                self.emit_expr(lhs);
                self.output.push_str(" = ");
                self.emit_expr(rhs);
            }
            Expr::Call(func, args) => {
                self.emit_expr(func);
                self.output.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { self.output.push_str(", "); }
                    let is_memcpy_ptr = matches!(func.as_ref(), Expr::Ident(name) if name == "memcpy" && i < 2);
                    if is_memcpy_ptr {
                        self.output.push_str("(uint8_t*)(");
                        self.emit_expr(arg);
                        self.output.push(')');
                    } else {
                        self.emit_expr(arg);
                    }
                }
                self.output.push(')');
            }
            Expr::Index(arr, index) => {
                let is_slice = self.resolve_expr_type(arr).map_or(false, |t| matches!(t, TypeVal::Slice(_)));
                if is_slice {
                    self.emit_expr(arr);
                    self.output.push_str(".ptr[");
                } else {
                    self.emit_expr(arr);
                    self.output.push('[');
                }
                self.emit_expr(index);
                self.output.push(']');
            }
            Expr::Field(obj, field) => {
                if self.expr_type_is_ptr(obj) {
                    self.output.push('(');
                    self.emit_expr(obj);
                    self.output.push(')');
                    self.output.push_str("->");
                } else {
                    self.emit_expr(obj);
                    self.output.push('.');
                }
                self.output.push_str(field);
            }
            Expr::Slice(arr, start, end) => {
                // For C backend, we need to emit a slice struct literal
                // This is a simplification
                self.emit_expr(arr);
                self.output.push_str(" /* slice");
                if let Some(s) = start {
                    self.output.push('[');
                    self.emit_expr(s);
                }
                self.output.push_str("..");
                if let Some(e) = end {
                    self.emit_expr(e);
                }
                self.output.push_str("] */");
            }
            Expr::StructInit(name, fields) => {
                self.output.push('(');
                self.emit_type_name(name);
                self.output.push(')');
                self.output.push('{');
                for (i, (_, val)) in fields.iter().enumerate() {
                    if i > 0 { self.output.push_str(", "); }
                    self.emit_expr(val);
                }
                self.output.push('}');
            }
            Expr::Comptime(inner) => {
                // comptime blocks evaluate at compile time; emit the inner expression.
                // If the inner expression is a block with a single statement, extract
                // just that expression so we don't emit invalid C like `{ 42; }` in an
                // initializer context.
                if let Expr::Block(block) = inner.as_ref() {
                    if block.stmts.len() == 1 {
                        if let Stmt::Expr(e) = &block.stmts[0] {
                            self.emit_expr(e);
                            return;
                        }
                    }
                }
                self.emit_expr(inner);
            }

            Expr::When(_cond, then_block, _else_branch) => {
                // when is compile-time branching; emit only the then branch as-if
                // (comptime evaluation selects the matching branch)
                self.emit_block_as_stmt(then_block);
            }

            Expr::ArrayLit(items) => {
                self.output.push('{');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { self.output.push_str(", "); }
                    self.emit_expr(item);
                }
                self.output.push('}');
            }
            Expr::Deref(inner) => {
                // x.*  →  (*x)  (parens allow correct field access like (*ptr).field)
                self.output.push_str("(*");
                self.emit_expr(inner);
                self.output.push(')');
            }
            Expr::Try(inner) => {
                // try expr  →  { auto _tryN = (expr); if (_tryN.err) return _tryN.err; _tryN.data.val; }
                let id = self.try_counter;
                self.try_counter += 1;
                self.output.push_str("({ auto _try");
                self.output.push_str(&id.to_string());
                self.output.push_str(" = (");
                self.emit_expr(inner);
                self.output.push_str("); if (_try");
                self.output.push_str(&id.to_string());
                self.output.push_str(".err) { ");
                self.emit_pending_defers();
                self.output.push_str("return _try");
                self.output.push_str(&id.to_string());
                self.output.push_str(".err; } _try");
                self.output.push_str(&id.to_string());
                self.output.push_str(".data.val; })");
            }
            Expr::Catch(lhs, rhs) => {
                // catch expr handler  →  ({ auto _tryN = (expr); _tryN.err ? (handler) : _tryN.data.val; })
                let id = self.try_counter;
                self.try_counter += 1;
                self.output.push_str("({ auto _try");
                self.output.push_str(&id.to_string());
                self.output.push_str(" = (");
                self.emit_expr(lhs);
                self.output.push_str("); _try");
                self.output.push_str(&id.to_string());
                self.output.push_str(".err ? (");
                self.emit_expr(rhs);
                self.output.push_str(") : _try");
                self.output.push_str(&id.to_string());
                self.output.push_str(".data.val; })");
            }
            Expr::OrElse(lhs, rhs) => {
                // opt orelse default
                let id = self.try_counter;
                self.try_counter += 1;
                self.output.push_str("({ auto _try");
                self.output.push_str(&id.to_string());
                self.output.push_str(" = (");
                self.emit_expr(lhs);
                self.output.push_str("); ");
                // Check if this is a nullable pointer (just T*) vs optional struct
                let is_ptr = match lhs.as_ref() {
                    Expr::Ident(name) => self.var_types.get(name)
                        .map(|t| matches!(t, TypeVal::NullablePtr(_) | TypeVal::ManyPtr(_)))
                        .unwrap_or(false),
                    _ => false,
                };
                if is_ptr {
                    self.output.push_str("_try");
                    self.output.push_str(&id.to_string());
                    self.output.push_str(" != NULL ? _try");
                    self.output.push_str(&id.to_string());
                    self.output.push_str(" : (");
                    self.emit_expr(rhs);
                    self.output.push_str("); })");
                } else {
                    // Optional struct: check .has
                    self.output.push_str("_try");
                    self.output.push_str(&id.to_string());
                    self.output.push_str(".has ? _try");
                    self.output.push_str(&id.to_string());
                    self.output.push_str(".val : (");
                    self.emit_expr(rhs);
                    self.output.push_str("); })");
                }
            }
        }
    }

    // === Helpers ===

    fn emit_block_as_stmt(&mut self, block: &Block) {
        self.emit_line(" {");
        self.indent += 1;
        self.defer_stack.push(Vec::new());
        for stmt in &block.stmts {
            self.emit_stmt(stmt);
        }
        self.emit_pending_defers();
        self.defer_stack.pop();
        self.indent -= 1;
        self.emit_indent();
        self.output.push('}');
    }

    fn emit_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }

    fn emit_line(&mut self, s: &str) {
        self.emit_indent();
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn type_is_ptr_type(t: &Type) -> bool {
        matches!(t, Type::Ptr(_) | Type::NullablePtr(_) | Type::ManyPtr(_) | Type::ConstPtr(_))
    }

    fn typeval_is_ptr(tv: &TypeVal) -> bool {
        matches!(tv, TypeVal::Ptr(_) | TypeVal::NullablePtr(_) | TypeVal::ManyPtr(_) | TypeVal::ConstPtr(_))
    }

    fn type_to_typeval(t: &Type) -> TypeVal {
        match t {
            Type::Named(name) => TypeVal::Named(name.clone()),
            Type::Ptr(inner) => TypeVal::Ptr(Box::new(Self::type_to_typeval(inner))),
            Type::ConstPtr(inner) => TypeVal::ConstPtr(Box::new(Self::type_to_typeval(inner))),
            Type::NullablePtr(inner) => TypeVal::NullablePtr(Box::new(Self::type_to_typeval(inner))),
            Type::ManyPtr(inner) => TypeVal::ManyPtr(Box::new(Self::type_to_typeval(inner))),
            Type::Slice(inner) => TypeVal::Slice(Box::new(Self::type_to_typeval(inner))),
            Type::Array(n, inner) => TypeVal::Array(*n, Box::new(Self::type_to_typeval(inner))),
            Type::Optional(inner) => TypeVal::Optional(Box::new(Self::type_to_typeval(inner))),
            Type::ErrorUnion(inner) => TypeVal::ErrorUnion(Box::new(Self::type_to_typeval(inner))),
            Type::Fn(params, ret) => TypeVal::Fn(
                params.iter().map(Self::type_to_typeval).collect(),
                ret.as_ref().map(|r| Box::new(Self::type_to_typeval(r)))
            ),
            Type::Inferred => unreachable!(),
        }
    }

    fn resolve_expr_type(&self, expr: &Expr) -> Option<TypeVal> {
        match expr {
            Expr::Ident(name) => self.var_types.get(name).cloned(),
            Expr::Field(obj, field) => {
                let mut obj_type = self.resolve_expr_type(obj)?;
                if Self::typeval_is_ptr(&obj_type) {
                    match &mut obj_type {
                        TypeVal::Ptr(t) | TypeVal::NullablePtr(t) | TypeVal::ManyPtr(t) | TypeVal::ConstPtr(t) => {
                            obj_type = *t.clone();
                        }
                        _ => {}
                    }
                }
                match &obj_type {
                    TypeVal::Struct(fields) => {
                        fields.iter().find(|(n, _)| n == field).map(|(_, ft)| ft.clone())
                    }
                    TypeVal::Named(n) => {
                        self.struct_fields.get(n).and_then(|fs| {
                            fs.iter().find(|(n, _)| n == field).map(|(_, ft)| Self::type_to_typeval(ft))
                        })
                    }
                    _ => None,
                }
            }
            Expr::Unary(UnOp::Deref, inner) => {
                let inner_type = self.resolve_expr_type(inner)?;
                match &inner_type {
                    TypeVal::Ptr(t) | TypeVal::NullablePtr(t) | TypeVal::ManyPtr(t) | TypeVal::ConstPtr(t) => Some(*t.clone()),
                    _ => None,
                }
            }
            Expr::Unary(UnOp::Addr, inner) => {
                let inner_type = self.resolve_expr_type(inner)?;
                Some(TypeVal::Ptr(Box::new(inner_type)))
            }
            Expr::Index(arr, _) => {
                let arr_type = self.resolve_expr_type(arr)?;
                match &arr_type {
                    TypeVal::Slice(elem) | TypeVal::Array(_, elem) => Some(*elem.clone()),
                    TypeVal::Ptr(elem) => Some(*elem.clone()),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn expr_type_is_ptr(&self, expr: &Expr) -> bool {
        self.resolve_expr_type(expr).map_or(false, |t| Self::typeval_is_ptr(&t))
    }
}

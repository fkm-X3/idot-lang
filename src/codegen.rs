use crate::parser;
use std::collections::HashMap;

pub struct CodeGen {
    output: String,
    val_counter: u64,
    label_counter: u64,
    named_values: HashMap<String, String>,
}

impl CodeGen {
    pub fn new() -> Self {
        let mut ir = String::new();
        ir.push_str("; ModuleID = 'main'\n");
        ir.push_str("target triple = \"x86_64-pc-windows-msvc\"\n");
        ir.push_str("\n");
        ir.push_str("declare i32 @printf(ptr, ...)\n");
        ir.push_str("@.printfmt = private unnamed_addr constant [6 x i8] c\"%lld\\0A\\00\"\n");
        ir.push_str("\n");
        CodeGen {
            output: ir,
            val_counter: 1,
            label_counter: 0,
            named_values: HashMap::new(),
        }
    }

    fn new_val(&mut self) -> String {
        let n = self.val_counter;
        self.val_counter += 1;
        format!("%{}", n)
    }

    fn new_label(&mut self) -> String {
        let n = self.label_counter;
        self.label_counter += 1;
        format!(".L{}", n)
    }

    pub fn compile(&mut self, program: &parser::Program) -> Result<String, String> {
        for func in &program.functions {
            self.compile_function(func)?;
        }
        Ok(self.output.clone())
    }

    fn compile_function(&mut self, func: &parser::Function) -> Result<(), String> {
        let params: Vec<String> = func
            .params
            .iter()
            .map(|p| format!("i64 %{}", p.name))
            .collect();
        self.output
            .push_str(&format!("define i64 @{}({}) {{\n", func.name, params.join(", ")));

        self.named_values.clear();

        for param in &func.params {
            let a = self.new_val();
            self.output.push_str(&format!("  {} = alloca i64\n", a));
            self.output
                .push_str(&format!("  store i64 %{}, ptr {}\n", param.name, a));
            self.named_values.insert(param.name.clone(), a);
        }

        self.compile_block(&func.body, 1)?;

        if !self.has_terminator() {
            self.output.push_str("  ret i64 0\n");
        }
        self.output.push_str("}\n\n");
        Ok(())
    }

    fn compile_block(&mut self, stmts: &[parser::Stmt], indent: usize) -> Result<(), String> {
        for stmt in stmts {
            self.compile_statement(stmt, indent)?;
        }
        Ok(())
    }

    fn has_terminator(&self) -> bool {
        let trimmed = self.output.trim_end();
        if let Some(last) = trimmed.lines().last() {
            let t = last.trim();
            t.starts_with("ret ") || t.starts_with("br ") || t.starts_with("unreachable")
        } else {
            false
        }
    }

    fn compile_statement(&mut self, stmt: &parser::Stmt, indent: usize) -> Result<(), String> {
        let p = "  ".repeat(indent);
        match stmt {
            parser::Stmt::Let { name, type_: _, init } => {
                let a = self.new_val();
                self.output.push_str(&format!("{} = alloca i64\n", a));
                if let Some(init_expr) = init {
                    let v = self.compile_expression(init_expr, indent)?;
                    self.output
                        .push_str(&format!("{}store i64 {}, ptr {}\n", p, v, a));
                }
                self.named_values.insert(name.clone(), a);
            }
            parser::Stmt::Expr(expr) => {
                self.compile_expression(expr, indent)?;
            }
            parser::Stmt::Return(Some(expr)) => {
                let v = self.compile_expression(expr, indent)?;
                self.output.push_str(&format!("{}ret i64 {}\n", p, v));
            }
            parser::Stmt::Return(None) => {
                self.output.push_str(&format!("{}ret i64 0\n", p));
            }
            parser::Stmt::If {
                cond,
                then_block,
                else_block,
            } => {
                let c = self.compile_expression(cond, indent)?;
                let cmp = self.new_val();
                self.output
                    .push_str(&format!("{} = icmp ne i64 {}, 0\n", cmp, c));

                let t_lab = self.new_label();
                let e_lab = self.new_label();
                let m_lab = self.new_label();

                self.output
                    .push_str(&format!("{}br i1 {}, label %{}, label %{}\n", p, cmp, t_lab, e_lab));
                self.output.push_str(&format!("\n{}:\n", t_lab));
                self.compile_block(then_block, indent)?;
                if !self.has_terminator() {
                    self.output
                        .push_str(&format!("{}br label %{}\n", p, m_lab));
                }

                self.output.push_str(&format!("\n{}:\n", e_lab));
                if let Some(eb) = else_block {
                    self.compile_block(eb, indent)?;
                }
                if !self.has_terminator() {
                    self.output
                        .push_str(&format!("{}br label %{}\n", p, m_lab));
                }

                self.output.push_str(&format!("\n{}:\n", m_lab));
            }
            parser::Stmt::While { cond, body } => {
                let c_lab = self.new_label();
                let b_lab = self.new_label();
                let a_lab = self.new_label();

                self.output
                    .push_str(&format!("{}br label %{}\n", p, c_lab));
                self.output.push_str(&format!("\n{}:\n", c_lab));

                let c = self.compile_expression(cond, indent)?;
                let cmp = self.new_val();
                self.output
                    .push_str(&format!("{} = icmp ne i64 {}, 0\n", cmp, c));
                self.output
                    .push_str(&format!("{}br i1 {}, label %{}, label %{}\n", p, cmp, b_lab, a_lab));

                self.output.push_str(&format!("\n{}:\n", b_lab));
                self.compile_block(body, indent)?;
                if !self.has_terminator() {
                    self.output
                        .push_str(&format!("{}br label %{}\n", p, c_lab));
                }

                self.output.push_str(&format!("\n{}:\n", a_lab));
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: &parser::Expr, indent: usize) -> Result<String, String> {
        let p = "  ".repeat(indent);
        match expr {
            parser::Expr::Integer(n) => Ok(format!("{}", n)),
            parser::Expr::Bool(b) => Ok(if *b { "1".to_string() } else { "0".to_string() }),
            parser::Expr::Ident(name) => {
                let alloc = self.named_values.get(name).cloned();
                if let Some(a) = alloc {
                    let v = self.new_val();
                    self.output
                        .push_str(&format!("{} = load i64, ptr {}\n", v, a));
                    Ok(v)
                } else {
                    Err(format!("undefined variable: {}", name))
                }
            }
            parser::Expr::Binary { op, lhs, rhs } => {
                let l = self.compile_expression(lhs, indent)?;
                let r = self.compile_expression(rhs, indent)?;
                let compare = matches!(op,
                    parser::BinOp::Eq | parser::BinOp::Ne |
                    parser::BinOp::Lt | parser::BinOp::Le |
                    parser::BinOp::Gt | parser::BinOp::Ge
                );
                if compare {
                    let cmp = self.new_val();
                    let res = self.new_val();
                    match op {
                        parser::BinOp::Eq => self.output.push_str(&format!("{} = icmp eq i64 {}, {}\n", cmp, l, r)),
                        parser::BinOp::Ne => self.output.push_str(&format!("{} = icmp ne i64 {}, {}\n", cmp, l, r)),
                        parser::BinOp::Lt => self.output.push_str(&format!("{} = icmp slt i64 {}, {}\n", cmp, l, r)),
                        parser::BinOp::Le => self.output.push_str(&format!("{} = icmp sle i64 {}, {}\n", cmp, l, r)),
                        parser::BinOp::Gt => self.output.push_str(&format!("{} = icmp sgt i64 {}, {}\n", cmp, l, r)),
                        parser::BinOp::Ge => self.output.push_str(&format!("{} = icmp sge i64 {}, {}\n", cmp, l, r)),
                        _ => unreachable!(),
                    }
                    self.output.push_str(&format!("{} = zext i1 {} to i64\n", res, cmp));
                    Ok(res)
                } else {
                    let res = self.new_val();
                    match op {
                        parser::BinOp::Add => self.output.push_str(&format!("{} = add i64 {}, {}\n", res, l, r)),
                        parser::BinOp::Sub => self.output.push_str(&format!("{} = sub i64 {}, {}\n", res, l, r)),
                        parser::BinOp::Mul => self.output.push_str(&format!("{} = mul i64 {}, {}\n", res, l, r)),
                        parser::BinOp::Div => self.output.push_str(&format!("{} = sdiv i64 {}, {}\n", res, l, r)),
                        _ => unreachable!(),
                    }
                    Ok(res)
                }
            }
            parser::Expr::Unary { op, operand } => {
                let v = self.compile_expression(operand, indent)?;
                let res = self.new_val();
                match op {
                    parser::UnOp::Neg => {
                        self.output
                            .push_str(&format!("{} = sub i64 0, {}\n", res, v))
                    }
                }
                Ok(res)
            }
            parser::Expr::Assign { name, value } => {
                let v = self.compile_expression(value, indent)?;
                if let Some(a) = self.named_values.get(name) {
                    self.output
                        .push_str(&format!("{}store i64 {}, ptr {}\n", p, v, a));
                    Ok(v)
                } else {
                    Err(format!("undefined variable: {}", name))
                }
            }
            parser::Expr::Call { name, args } => {
                if name == "print" {
                    return self.compile_print(args, indent);
                }

                let mut arg_vals = Vec::new();
                for arg in args {
                    let v = self.compile_expression(arg, indent)?;
                    arg_vals.push(v);
                }
                let a: Vec<String> = arg_vals.iter().map(|v| format!("i64 {}", v)).collect();
                let res = self.new_val();
                self.output
                    .push_str(&format!("{} = call i64 @{}({})\n", res, name, a.join(", ")));
                Ok(res)
            }
        }
    }

    fn compile_print(&mut self, args: &[parser::Expr], indent: usize) -> Result<String, String> {
        for arg in args {
            let v = self.compile_expression(arg, indent)?;
            let fmt = self.new_val();
            self.output
                .push_str(&format!("{} = getelementptr [6 x i8], ptr @.printfmt, i64 0, i64 0\n", fmt));
            let call = self.new_val();
            self.output
                .push_str(&format!("{} = call i32 (ptr, ...) @printf(ptr {}, i64 {})\n", call, fmt, v));
        }
        Ok("0".to_string())
    }
}

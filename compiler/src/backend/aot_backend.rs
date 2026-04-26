use std::collections::HashMap;

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::{AbiParam, FuncRef, InstBuilder, Type, Value as IrValue};
use cranelift_codegen::settings;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::ast::{Expr, Stmt};
use crate::diagnostics::{DiagnosticError, ErrorPhase, Result};
use crate::token::{Token, TokenType};
use crate::value::Value as RuntimeValue;

pub fn compile_aot(statements: &[Stmt], output_path: &str) -> Result<()> {
    let isa_builder = cranelift_native::builder()
        .map_err(|error| backend_error(format!("Unsupported host ISA: {error}")))?;
    let flags = settings::Flags::new(settings::builder());
    let isa = isa_builder
        .finish(flags)
        .map_err(|error| backend_error(format!("Failed to configure host ISA: {error}")))?;

    let builder = ObjectBuilder::new(isa, "idot_module", cranelift_module::default_libcall_names())
        .map_err(|e| backend_error(format!("Failed to create object builder: {e}")))?;
    let mut module = ObjectModule::new(builder);

    let pointer_type = module.target_config().pointer_type();
    let runtime_ids = RuntimeFuncIds::declare(&mut module, pointer_type)?;

    let mut ctx = module.make_context();
    let mut func_ctx = FunctionBuilderContext::new();

    let mut signature = module.make_signature();
    signature.returns.push(AbiParam::new(types::I32));
    ctx.func.signature = signature;

    let function_id = module
        .declare_function("idot_main", Linkage::Export, &ctx.func.signature)
        .map_err(module_error)?;

    {
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let entry = builder.create_block();
        builder.switch_to_block(entry);
        builder.seal_block(entry);

        let runtime_refs =
            RuntimeFuncRefs::declare_in_function(&mut module, builder.func, &runtime_ids);
        let mut codegen = CodeGenerator::new(&mut module, builder, runtime_refs, pointer_type);

        for statement in statements {
            codegen.emit_stmt(statement)?;
        }

        let zero = codegen.builder.ins().iconst(types::I32, 0);
        codegen.builder.ins().return_(&[zero]);
        codegen.builder.finalize();
    }

    module
        .define_function(function_id, &mut ctx)
        .map_err(module_error)?;
    module.clear_context(&mut ctx);

    let product = module.finish();
    std::fs::write(output_path, product.emit().map_err(|e| {
        backend_error(format!("Failed to emit object file: {e}"))
    })?)
        .map_err(|e| backend_error(format!("Failed to write object file: {e}")))?;

    Ok(())
}

pub fn execute_and_return_output(statements: &[Stmt]) -> Result<String> {
    let mut output = Vec::new();
    crate::interpreter::Interpreter::new().execute(statements, &mut output)?;
    Ok(String::from_utf8(output).unwrap_or_else(|_| "Invalid UTF-8 output".to_string()))
}


struct CodeGenerator<'module, 'func> {
    module: &'module mut ObjectModule,
    builder: FunctionBuilder<'func>,
    runtime: RuntimeFuncRefs,
    pointer_type: Type,
    scopes: Vec<HashMap<String, Variable>>,
    next_data_symbol: usize,
}

impl<'module, 'func> CodeGenerator<'module, 'func> {
    fn new(
        module: &'module mut ObjectModule,
        builder: FunctionBuilder<'func>,
        runtime: RuntimeFuncRefs,
        pointer_type: Type,
    ) -> Self {
        Self {
            module,
            builder,
            runtime,
            pointer_type,
            scopes: vec![HashMap::new()],
            next_data_symbol: 0,
        }
    }

    fn emit_stmt(&mut self, statement: &Stmt) -> Result<()> {
        match statement {
            Stmt::Block(statements) => {
                self.push_scope();
                for statement in statements {
                    self.emit_stmt(statement)?;
                }
                self.pop_scope();
            }
            Stmt::Expr(expression) => {
                let _ = self.emit_expr(expression)?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.emit_if_stmt(condition, then_branch, else_branch.as_deref())?;
            }
            Stmt::Print(expression) => {
                let value = self.emit_expr(expression)?;
                self.call_runtime1_void(self.runtime.print, value);
            }
            Stmt::Var { name, initializer } => {
                let value = self.emit_expr(initializer)?;
                let variable = self.builder.declare_var(types::I64);
                self.builder.def_var(variable, value);
                self.current_scope_mut()
                    .insert(name.lexeme.clone(), variable);
            }
        }
        Ok(())
    }

    fn emit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
    ) -> Result<()> {
        let then_block = self.builder.create_block();
        let else_block = self.builder.create_block();
        let merge_block = self.builder.create_block();

        let condition_value = self.emit_expr(condition)?;
        let truthy = self.call_runtime1(self.runtime.is_truthy, condition_value);
        let branch_condition = self.builder.ins().icmp_imm(IntCC::NotEqual, truthy, 0);
        self.builder
            .ins()
            .brif(branch_condition, then_block, &[], else_block, &[]);

        self.builder.switch_to_block(then_block);
        self.emit_stmt(then_branch)?;
        self.builder.ins().jump(merge_block, &[]);
        self.builder.seal_block(then_block);

        self.builder.switch_to_block(else_block);
        if let Some(else_branch) = else_branch {
            self.emit_stmt(else_branch)?;
        }
        self.builder.ins().jump(merge_block, &[]);
        self.builder.seal_block(else_block);

        self.builder.switch_to_block(merge_block);
        self.builder.seal_block(merge_block);
        Ok(())
    }

    fn emit_expr(&mut self, expression: &Expr) -> Result<IrValue> {
        match expression {
            Expr::Assign { name, value } => {
                let assigned = self.emit_expr(value)?;
                let variable = self
                    .lookup_var(&name.lexeme)
                    .ok_or_else(|| undefined_variable(name))?;
                self.builder.def_var(variable, assigned);
                Ok(assigned)
            }
            Expr::Binary { left, op, right } => {
                let left = self.emit_expr(left)?;
                let right = self.emit_expr(right)?;
                let runtime_op = match op.kind {
                    TokenType::Plus => self.runtime.add,
                    TokenType::Minus => self.runtime.sub,
                    TokenType::Star => self.runtime.mul,
                    TokenType::Slash => self.runtime.div,
                    TokenType::Percent => self.runtime.modulo,
                    TokenType::EqualEqual => self.runtime.eq,
                    TokenType::BangEqual => self.runtime.neq,
                    TokenType::Less => self.runtime.lt,
                    TokenType::LessEqual => self.runtime.lte,
                    TokenType::Greater => self.runtime.gt,
                    TokenType::GreaterEqual => self.runtime.gte,
                    _ => {
                        return Err(DiagnosticError::new(
                            ErrorPhase::Runtime,
                            op.line,
                            op.column,
                            "Unsupported binary operator in AOT backend.",
                        ));
                    }
                };
                Ok(self.call_runtime2(runtime_op, left, right))
            }
            Expr::Call { .. } => {
                Err(DiagnosticError::new(
                    ErrorPhase::Runtime,
                    0,
                    0,
                    "Function calls are not yet supported in AOT backend.",
                ))
            }
            Expr::Grouping(inner) => self.emit_expr(inner),
            Expr::Literal(value) => match value {
                RuntimeValue::Nil => Ok(self.call_runtime0(self.runtime.nil)),
                RuntimeValue::Number(number) => {
                    let literal = self.builder.ins().f64const(*number);
                    Ok(self.call_runtime1(self.runtime.num, literal))
                }
                RuntimeValue::Bool(boolean) => {
                    let value = self
                        .builder
                        .ins()
                        .iconst(types::I64, if *boolean { 1 } else { 0 });
                    Ok(self.call_runtime1(self.runtime.bool_, value))
                }
                RuntimeValue::String(text) => self.emit_string_literal(text),
            },
            Expr::Unary { op, right } => {
                let right = self.emit_expr(right)?;
                let runtime_op = match op.kind {
                    TokenType::Bang => self.runtime.not,
                    TokenType::Minus => self.runtime.neg,
                    _ => {
                        return Err(DiagnosticError::new(
                            ErrorPhase::Runtime,
                            op.line,
                            op.column,
                            "Unsupported unary operator in AOT backend.",
                        ));
                    }
                };
                Ok(self.call_runtime1(runtime_op, right))
            }
            Expr::Variable(name) => {
                let variable = self
                    .lookup_var(&name.lexeme)
                    .ok_or_else(|| undefined_variable(name))?;
                Ok(self.builder.use_var(variable))
            }
        }
    }

    fn emit_string_literal(&mut self, text: &str) -> Result<IrValue> {
        let symbol_name = format!("idot_literal_{}", self.next_data_symbol);
        self.next_data_symbol += 1;

        let data_id = self
            .module
            .declare_data(&symbol_name, Linkage::Local, false, false)
            .map_err(module_error)?;
        let mut data = DataDescription::new();
        data.define(text.as_bytes().to_vec().into_boxed_slice());
        self.module
            .define_data(data_id, &data)
            .map_err(module_error)?;

        let local_data = self.module.declare_data_in_func(data_id, self.builder.func);
        let ptr = self
            .builder
            .ins()
            .global_value(self.pointer_type, local_data);
        let len = self.builder.ins().iconst(types::I64, text.len() as i64);
        Ok(self.call_runtime2(self.runtime.str_, ptr, len))
    }

    fn call_runtime0(&mut self, function: FuncRef) -> IrValue {
        let call = self.builder.ins().call(function, &[]);
        self.builder.inst_results(call)[0]
    }

    fn call_runtime1(&mut self, function: FuncRef, arg0: IrValue) -> IrValue {
        let call = self.builder.ins().call(function, &[arg0]);
        self.builder.inst_results(call)[0]
    }

    fn call_runtime2(&mut self, function: FuncRef, arg0: IrValue, arg1: IrValue) -> IrValue {
        let call = self.builder.ins().call(function, &[arg0, arg1]);
        self.builder.inst_results(call)[0]
    }

    fn call_runtime1_void(&mut self, function: FuncRef, arg0: IrValue) {
        let _ = self.builder.ins().call(function, &[arg0]);
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<String, Variable> {
        self.scopes
            .last_mut()
            .expect("AOT backend should always have a scope")
    }

    fn lookup_var(&self, name: &str) -> Option<Variable> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }
}

struct RuntimeFuncIds {
    nil: FuncId,
    num: FuncId,
    bool_: FuncId,
    str_: FuncId,
    add: FuncId,
    sub: FuncId,
    mul: FuncId,
    div: FuncId,
    modulo: FuncId,
    eq: FuncId,
    neq: FuncId,
    lt: FuncId,
    lte: FuncId,
    gt: FuncId,
    gte: FuncId,
    not: FuncId,
    neg: FuncId,
    is_truthy: FuncId,
    print: FuncId,
}

impl RuntimeFuncIds {
    fn declare(module: &mut ObjectModule, pointer_type: Type) -> Result<Self> {
        Ok(Self {
            nil: declare_import(module, "idot_rt_nil", &[], &[types::I64])?,
            num: declare_import(module, "idot_rt_num", &[types::F64], &[types::I64])?,
            bool_: declare_import(module, "idot_rt_bool", &[types::I64], &[types::I64])?,
            str_: declare_import(
                module,
                "idot_rt_str",
                &[pointer_type, types::I64],
                &[types::I64],
            )?,
            add: declare_import(
                module,
                "idot_rt_add",
                &[types::I64, types::I64],
                &[types::I64],
            )?,
            sub: declare_import(
                module,
                "idot_rt_sub",
                &[types::I64, types::I64],
                &[types::I64],
            )?,
            mul: declare_import(
                module,
                "idot_rt_mul",
                &[types::I64, types::I64],
                &[types::I64],
            )?,
            div: declare_import(
                module,
                "idot_rt_div",
                &[types::I64, types::I64],
                &[types::I64],
            )?,
            modulo: declare_import(
                module,
                "idot_rt_mod",
                &[types::I64, types::I64],
                &[types::I64],
            )?,
            eq: declare_import(
                module,
                "idot_rt_eq",
                &[types::I64, types::I64],
                &[types::I64],
            )?,
            neq: declare_import(
                module,
                "idot_rt_neq",
                &[types::I64, types::I64],
                &[types::I64],
            )?,
            lt: declare_import(
                module,
                "idot_rt_lt",
                &[types::I64, types::I64],
                &[types::I64],
            )?,
            lte: declare_import(
                module,
                "idot_rt_lte",
                &[types::I64, types::I64],
                &[types::I64],
            )?,
            gt: declare_import(
                module,
                "idot_rt_gt",
                &[types::I64, types::I64],
                &[types::I64],
            )?,
            gte: declare_import(
                module,
                "idot_rt_gte",
                &[types::I64, types::I64],
                &[types::I64],
            )?,
            not: declare_import(module, "idot_rt_not", &[types::I64], &[types::I64])?,
            neg: declare_import(module, "idot_rt_neg", &[types::I64], &[types::I64])?,
            is_truthy: declare_import(module, "idot_rt_is_truthy", &[types::I64], &[types::I64])?,
            print: declare_import(module, "idot_rt_print", &[types::I64], &[])?,
        })
    }
}

struct RuntimeFuncRefs {
    nil: FuncRef,
    num: FuncRef,
    bool_: FuncRef,
    str_: FuncRef,
    add: FuncRef,
    sub: FuncRef,
    mul: FuncRef,
    div: FuncRef,
    modulo: FuncRef,
    eq: FuncRef,
    neq: FuncRef,
    lt: FuncRef,
    lte: FuncRef,
    gt: FuncRef,
    gte: FuncRef,
    not: FuncRef,
    neg: FuncRef,
    is_truthy: FuncRef,
    print: FuncRef,
}

impl RuntimeFuncRefs {
    fn declare_in_function(
        module: &mut ObjectModule,
        function: &mut cranelift_codegen::ir::Function,
        ids: &RuntimeFuncIds,
    ) -> Self {
        Self {
            nil: module.declare_func_in_func(ids.nil, function),
            num: module.declare_func_in_func(ids.num, function),
            bool_: module.declare_func_in_func(ids.bool_, function),
            str_: module.declare_func_in_func(ids.str_, function),
            add: module.declare_func_in_func(ids.add, function),
            sub: module.declare_func_in_func(ids.sub, function),
            mul: module.declare_func_in_func(ids.mul, function),
            div: module.declare_func_in_func(ids.div, function),
            modulo: module.declare_func_in_func(ids.modulo, function),
            eq: module.declare_func_in_func(ids.eq, function),
            neq: module.declare_func_in_func(ids.neq, function),
            lt: module.declare_func_in_func(ids.lt, function),
            lte: module.declare_func_in_func(ids.lte, function),
            gt: module.declare_func_in_func(ids.gt, function),
            gte: module.declare_func_in_func(ids.gte, function),
            not: module.declare_func_in_func(ids.not, function),
            neg: module.declare_func_in_func(ids.neg, function),
            is_truthy: module.declare_func_in_func(ids.is_truthy, function),
            print: module.declare_func_in_func(ids.print, function),
        }
    }
}

fn declare_import(
    module: &mut ObjectModule,
    name: &str,
    params: &[Type],
    returns: &[Type],
) -> Result<FuncId> {
    let mut signature = module.make_signature();
    for param in params {
        signature.params.push(AbiParam::new(*param));
    }
    for value in returns {
        signature.returns.push(AbiParam::new(*value));
    }
    module
        .declare_function(name, Linkage::Import, &signature)
        .map_err(module_error)
}

fn module_error(error: impl std::fmt::Display) -> DiagnosticError {
    backend_error(format!("AOT backend error: {error}"))
}

fn backend_error(message: impl Into<String>) -> DiagnosticError {
    DiagnosticError::new(ErrorPhase::Runtime, 0, 0, message)
}

fn undefined_variable(name: &Token) -> DiagnosticError {
    DiagnosticError::new(
        ErrorPhase::Runtime,
        name.line,
        name.column,
        format!("Undefined variable '{}'.", name.lexeme),
    )
}

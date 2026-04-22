use std::collections::HashMap;

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::{AbiParam, FuncRef, InstBuilder, Type, Value as IrValue};
use cranelift_codegen::settings;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, DataDescription, FuncId, Linkage, Module};

use crate::ast::{Expr, Stmt};
use crate::diagnostics::{DiagnosticError, ErrorPhase, Result};
use crate::token::{Token, TokenType};
use crate::value::Value as RuntimeValue;

pub fn run_native(statements: &[Stmt]) -> Result<()> {
    let isa_builder = cranelift_native::builder()
        .map_err(|error| backend_error(format!("Unsupported host ISA: {error}")))?;
    let flags = settings::Flags::new(settings::builder());
    let isa = isa_builder
        .finish(flags)
        .map_err(|error| backend_error(format!("Failed to configure host ISA: {error}")))?;

    let mut jit_builder = JITBuilder::with_isa(isa, default_libcall_names());
    register_runtime_symbols(&mut jit_builder);

    let mut module = JITModule::new(jit_builder);
    let pointer_type = module.target_config().pointer_type();
    let runtime_ids = RuntimeFuncIds::declare(&mut module, pointer_type)?;

    let mut ctx = module.make_context();
    let mut func_ctx = FunctionBuilderContext::new();

    let mut signature = module.make_signature();
    signature.returns.push(AbiParam::new(types::I32));
    ctx.func.signature = signature;

    let function_id = module
        .declare_function("idot_main", Linkage::Local, &ctx.func.signature)
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
    module.finalize_definitions().map_err(module_error)?;

    let function_ptr = module.get_finalized_function(function_id);
    let callable =
        unsafe { std::mem::transmute::<*const u8, extern "C" fn() -> i32>(function_ptr) };
    let _ = callable();
    Ok(())
}

struct CodeGenerator<'module, 'func> {
    module: &'module mut JITModule,
    builder: FunctionBuilder<'func>,
    runtime: RuntimeFuncRefs,
    pointer_type: Type,
    scopes: Vec<HashMap<String, Variable>>,
    next_data_symbol: usize,
}

impl<'module, 'func> CodeGenerator<'module, 'func> {
    fn new(
        module: &'module mut JITModule,
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
                            "Unsupported binary operator in native backend.",
                        ));
                    }
                };
                Ok(self.call_runtime2(runtime_op, left, right))
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
                            "Unsupported unary operator in native backend.",
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
            .expect("native backend should always have a scope")
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
    fn declare(module: &mut JITModule, pointer_type: Type) -> Result<Self> {
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
        module: &mut JITModule,
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
    module: &mut JITModule,
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

fn register_runtime_symbols(builder: &mut JITBuilder) {
    builder.symbol("idot_rt_nil", idot_rt_nil as *const u8);
    builder.symbol("idot_rt_num", idot_rt_num as *const u8);
    builder.symbol("idot_rt_bool", idot_rt_bool as *const u8);
    builder.symbol("idot_rt_str", idot_rt_str as *const u8);
    builder.symbol("idot_rt_add", idot_rt_add as *const u8);
    builder.symbol("idot_rt_sub", idot_rt_sub as *const u8);
    builder.symbol("idot_rt_mul", idot_rt_mul as *const u8);
    builder.symbol("idot_rt_div", idot_rt_div as *const u8);
    builder.symbol("idot_rt_mod", idot_rt_mod as *const u8);
    builder.symbol("idot_rt_eq", idot_rt_eq as *const u8);
    builder.symbol("idot_rt_neq", idot_rt_neq as *const u8);
    builder.symbol("idot_rt_lt", idot_rt_lt as *const u8);
    builder.symbol("idot_rt_lte", idot_rt_lte as *const u8);
    builder.symbol("idot_rt_gt", idot_rt_gt as *const u8);
    builder.symbol("idot_rt_gte", idot_rt_gte as *const u8);
    builder.symbol("idot_rt_not", idot_rt_not as *const u8);
    builder.symbol("idot_rt_neg", idot_rt_neg as *const u8);
    builder.symbol("idot_rt_is_truthy", idot_rt_is_truthy as *const u8);
    builder.symbol("idot_rt_print", idot_rt_print as *const u8);
}

extern "C" fn idot_rt_nil() -> u64 {
    box_value(RuntimeValue::Nil)
}

extern "C" fn idot_rt_num(number: f64) -> u64 {
    box_value(RuntimeValue::Number(number))
}

extern "C" fn idot_rt_bool(boolean: u64) -> u64 {
    box_value(RuntimeValue::Bool(boolean != 0))
}

extern "C" fn idot_rt_str(ptr: *const u8, len: i64) -> u64 {
    if ptr.is_null() || len < 0 {
        runtime_error("Invalid string literal.");
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    let text =
        std::str::from_utf8(slice).unwrap_or_else(|_| runtime_error("Invalid UTF-8 string."));
    box_value(RuntimeValue::String(text.to_string()))
}

extern "C" fn idot_rt_add(left: u64, right: u64) -> u64 {
    match (as_value(left), as_value(right)) {
        (RuntimeValue::Number(left), RuntimeValue::Number(right)) => {
            box_value(RuntimeValue::Number(left + right))
        }
        (RuntimeValue::String(left), RuntimeValue::String(right)) => {
            box_value(RuntimeValue::String(format!("{left}{right}")))
        }
        _ => runtime_error("Operator '+' requires two numbers or two strings."),
    }
}

extern "C" fn idot_rt_sub(left: u64, right: u64) -> u64 {
    box_value(RuntimeValue::Number(
        require_number(left, "left operand") - require_number(right, "right operand"),
    ))
}

extern "C" fn idot_rt_mul(left: u64, right: u64) -> u64 {
    box_value(RuntimeValue::Number(
        require_number(left, "left operand") * require_number(right, "right operand"),
    ))
}

extern "C" fn idot_rt_div(left: u64, right: u64) -> u64 {
    let divisor = require_number(right, "right operand");
    if divisor == 0.0 {
        runtime_error("Division by zero.");
    }
    box_value(RuntimeValue::Number(
        require_number(left, "left operand") / divisor,
    ))
}

extern "C" fn idot_rt_mod(left: u64, right: u64) -> u64 {
    let divisor = require_number(right, "right operand");
    if divisor == 0.0 {
        runtime_error("Modulo by zero.");
    }
    box_value(RuntimeValue::Number(
        require_number(left, "left operand") % divisor,
    ))
}

extern "C" fn idot_rt_eq(left: u64, right: u64) -> u64 {
    box_value(RuntimeValue::Bool(as_value(left) == as_value(right)))
}

extern "C" fn idot_rt_neq(left: u64, right: u64) -> u64 {
    box_value(RuntimeValue::Bool(as_value(left) != as_value(right)))
}

extern "C" fn idot_rt_lt(left: u64, right: u64) -> u64 {
    box_value(RuntimeValue::Bool(
        require_number(left, "left operand") < require_number(right, "right operand"),
    ))
}

extern "C" fn idot_rt_lte(left: u64, right: u64) -> u64 {
    box_value(RuntimeValue::Bool(
        require_number(left, "left operand") <= require_number(right, "right operand"),
    ))
}

extern "C" fn idot_rt_gt(left: u64, right: u64) -> u64 {
    box_value(RuntimeValue::Bool(
        require_number(left, "left operand") > require_number(right, "right operand"),
    ))
}

extern "C" fn idot_rt_gte(left: u64, right: u64) -> u64 {
    box_value(RuntimeValue::Bool(
        require_number(left, "left operand") >= require_number(right, "right operand"),
    ))
}

extern "C" fn idot_rt_not(value: u64) -> u64 {
    box_value(RuntimeValue::Bool(!as_value(value).is_truthy()))
}

extern "C" fn idot_rt_neg(value: u64) -> u64 {
    box_value(RuntimeValue::Number(-require_number(value, "operand")))
}

extern "C" fn idot_rt_is_truthy(value: u64) -> u64 {
    if as_value(value).is_truthy() {
        1
    } else {
        0
    }
}

extern "C" fn idot_rt_print(value: u64) {
    println!("{}", as_value(value).to_idot_string());
}

fn box_value(value: RuntimeValue) -> u64 {
    Box::into_raw(Box::new(value)) as u64
}

fn as_value(handle: u64) -> &'static RuntimeValue {
    if handle == 0 {
        runtime_error("Invalid runtime value handle.");
    }
    unsafe { &*(handle as *const RuntimeValue) }
}

fn require_number(handle: u64, context: &str) -> f64 {
    match as_value(handle) {
        RuntimeValue::Number(number) => *number,
        _ => runtime_error(&format!("Expected number for {context}.")),
    }
}

fn runtime_error(message: &str) -> ! {
    eprintln!("Runtime error: {message}");
    std::process::exit(1);
}

fn module_error(error: impl std::fmt::Display) -> DiagnosticError {
    backend_error(format!("Native backend error: {error}"))
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

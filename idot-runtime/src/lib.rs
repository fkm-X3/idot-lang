#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Number(f64),
    Bool(bool),
    String(String),
}

impl Value {
    pub fn to_idot_string(&self) -> String {
        match self {
            Self::Nil => "nil".to_string(),
            Self::Bool(value) => {
                if *value {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Self::Number(value) => {
                let mut text = format!("{value:.15}");
                while text.contains('.') && text.ends_with('0') {
                    text.pop();
                }
                if text.ends_with('.') {
                    text.pop();
                }
                text
            }
            Self::String(value) => value.clone(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Nil => false,
            Self::Bool(value) => *value,
            _ => true,
        }
    }
}

// Runtime functions exposed via C ABI for use by generated code

#[no_mangle]
pub extern "C" fn idot_rt_nil() -> u64 {
    box_value(Value::Nil)
}

#[no_mangle]
pub extern "C" fn idot_rt_num(number: f64) -> u64 {
    box_value(Value::Number(number))
}

#[no_mangle]
pub extern "C" fn idot_rt_bool(boolean: u64) -> u64 {
    box_value(Value::Bool(boolean != 0))
}

#[no_mangle]
pub extern "C" fn idot_rt_str(ptr: *const u8, len: i64) -> u64 {
    if ptr.is_null() || len < 0 {
        runtime_error("Invalid string literal.");
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    let text =
        std::str::from_utf8(slice).unwrap_or_else(|_| runtime_error("Invalid UTF-8 string."));
    box_value(Value::String(text.to_string()))
}

#[no_mangle]
pub extern "C" fn idot_rt_add(left: u64, right: u64) -> u64 {
    match (as_value(left), as_value(right)) {
        (Value::Number(left), Value::Number(right)) => {
            box_value(Value::Number(left + right))
        }
        (Value::String(left), Value::String(right)) => {
            box_value(Value::String(format!("{left}{right}")))
        }
        _ => runtime_error("Operator '+' requires two numbers or two strings."),
    }
}

#[no_mangle]
pub extern "C" fn idot_rt_sub(left: u64, right: u64) -> u64 {
    box_value(Value::Number(
        require_number(left, "left operand") - require_number(right, "right operand"),
    ))
}

#[no_mangle]
pub extern "C" fn idot_rt_mul(left: u64, right: u64) -> u64 {
    box_value(Value::Number(
        require_number(left, "left operand") * require_number(right, "right operand"),
    ))
}

#[no_mangle]
pub extern "C" fn idot_rt_div(left: u64, right: u64) -> u64 {
    let divisor = require_number(right, "right operand");
    if divisor == 0.0 {
        runtime_error("Division by zero.");
    }
    box_value(Value::Number(
        require_number(left, "left operand") / divisor,
    ))
}

#[no_mangle]
pub extern "C" fn idot_rt_mod(left: u64, right: u64) -> u64 {
    let divisor = require_number(right, "right operand");
    if divisor == 0.0 {
        runtime_error("Modulo by zero.");
    }
    box_value(Value::Number(
        require_number(left, "left operand") % divisor,
    ))
}

#[no_mangle]
pub extern "C" fn idot_rt_eq(left: u64, right: u64) -> u64 {
    box_value(Value::Bool(as_value(left) == as_value(right)))
}

#[no_mangle]
pub extern "C" fn idot_rt_neq(left: u64, right: u64) -> u64 {
    box_value(Value::Bool(as_value(left) != as_value(right)))
}

#[no_mangle]
pub extern "C" fn idot_rt_lt(left: u64, right: u64) -> u64 {
    box_value(Value::Bool(
        require_number(left, "left operand") < require_number(right, "right operand"),
    ))
}

#[no_mangle]
pub extern "C" fn idot_rt_lte(left: u64, right: u64) -> u64 {
    box_value(Value::Bool(
        require_number(left, "left operand") <= require_number(right, "right operand"),
    ))
}

#[no_mangle]
pub extern "C" fn idot_rt_gt(left: u64, right: u64) -> u64 {
    box_value(Value::Bool(
        require_number(left, "left operand") > require_number(right, "right operand"),
    ))
}

#[no_mangle]
pub extern "C" fn idot_rt_gte(left: u64, right: u64) -> u64 {
    box_value(Value::Bool(
        require_number(left, "left operand") >= require_number(right, "right operand"),
    ))
}

#[no_mangle]
pub extern "C" fn idot_rt_not(value: u64) -> u64 {
    box_value(Value::Bool(!as_value(value).is_truthy()))
}

#[no_mangle]
pub extern "C" fn idot_rt_neg(value: u64) -> u64 {
    box_value(Value::Number(-require_number(value, "operand")))
}

#[no_mangle]
pub extern "C" fn idot_rt_is_truthy(value: u64) -> u64 {
    if as_value(value).is_truthy() {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn idot_rt_print(value: u64) {
    println!("{}", as_value(value).to_idot_string());
}

fn box_value(value: Value) -> u64 {
    Box::into_raw(Box::new(value)) as u64
}

fn as_value(handle: u64) -> &'static Value {
    if handle == 0 {
        runtime_error("Invalid runtime value handle.");
    }
    unsafe { &*(handle as *const Value) }
}

fn require_number(handle: u64, context: &str) -> f64 {
    match as_value(handle) {
        Value::Number(number) => *number,
        _ => runtime_error(&format!("Expected number for {context}.")),
    }
}

fn runtime_error(message: &str) -> ! {
    eprintln!("Runtime error: {message}");
    std::process::exit(1);
}


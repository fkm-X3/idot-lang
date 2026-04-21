use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::diagnostics::{DiagnosticError, ErrorPhase, Result};
use crate::token::Token;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<()> {
        if let Some(slot) = self.values.get_mut(&name.lexeme) {
            *slot = value;
            return Ok(());
        }

        if let Some(enclosing) = self.enclosing.as_ref().cloned() {
            return enclosing.borrow_mut().assign(name, value);
        }

        Err(DiagnosticError::new(
            ErrorPhase::Runtime,
            name.line,
            name.column,
            format!("Undefined variable '{}'.", name.lexeme),
        ))
    }

    pub fn get(&self, name: &Token) -> Result<Value> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value.clone());
        }

        if let Some(enclosing) = self.enclosing.as_ref().cloned() {
            return enclosing.borrow().get(name);
        }

        Err(DiagnosticError::new(
            ErrorPhase::Runtime,
            name.line,
            name.column,
            format!("Undefined variable '{}'.", name.lexeme),
        ))
    }
}

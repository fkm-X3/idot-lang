use std::io::Write;

use crate::diagnostics::Result;
use crate::interpreter::Interpreter;
use crate::{lexer, parser};

pub struct Session {
    interpreter: Interpreter,
}

impl Session {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    pub fn execute(&mut self, source: &str, output: &mut dyn Write) -> Result<()> {
        let tokens = lexer::scan_tokens(source)?;
        let statements = parser::parse(tokens)?;
        self.interpreter.execute(&statements, output)
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

pub fn execute_source_to_string(source: &str) -> Result<String> {
    let mut session = Session::new();
    let mut output = Vec::new();
    session.execute(source, &mut output)?;
    Ok(String::from_utf8(output).expect("runtime output must be valid UTF-8"))
}

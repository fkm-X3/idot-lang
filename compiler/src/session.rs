use std::path::Path;

use crate::backend::aot_backend;
use crate::diagnostics::Result;
use crate::{lexer, parser};

pub struct Session;

impl Session {
    pub fn compile_aot<P: AsRef<Path>>(source: &str, output_path: P) -> Result<()> {
        let tokens = lexer::scan_tokens(source)?;
        let statements = parser::parse(tokens)?;
        aot_backend::compile_aot(&statements, output_path.as_ref().to_str().unwrap())
    }
}

pub fn execute_source_to_string(source: &str) -> Result<String> {
    let tokens = lexer::scan_tokens(source)?;
    let statements = parser::parse(tokens)?;
    aot_backend::execute_and_return_output(&statements)
}


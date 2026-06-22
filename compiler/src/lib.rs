pub mod ast;
pub mod lexer;
pub mod parser;
pub mod codegen;

use codegen::c::CBackend;
use parser::Parser;

pub fn compile(source: &str) -> String {
    let mut parser = Parser::new(source);
    let program = parser.parse();
    let mut backend = CBackend::new();
    backend.generate(&program)
}

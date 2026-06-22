pub mod ast;
pub mod lexer;
pub mod parser;
pub mod codegen;
pub mod semantic;

use codegen::c::CBackend;
use parser::Parser;
use semantic::SemanticAnalyzer;

pub fn compile(source: &str) -> String {
    let mut parser = Parser::new(source);
    let mut program = parser.parse();

    let mut analyzer = SemanticAnalyzer::new();
    if let Err(errors) = analyzer.analyze(&mut program) {
        for err in &errors {
            eprintln!("semantic error: {}", err);
        }
        eprintln!("Compilation aborted due to {} semantic error(s)", errors.len());
        std::process::exit(1);
    }

    let mut backend = CBackend::new();
    backend.generate(&program)
}


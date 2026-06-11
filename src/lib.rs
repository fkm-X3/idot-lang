pub mod lexer;
pub mod parser;
pub mod codegen;

use std::fs;
use std::path::Path;
use std::process;

pub fn compile_source(source: &str) -> Result<String, Vec<String>> {
    let mut lexer = lexer::Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program();

    if !parser.errors.is_empty() {
        return Err(parser.errors);
    }

    let mut codegen = codegen::CodeGen::new();
    codegen.compile(&program).map_err(|e| vec![e])
}

pub fn compile_to_exe(source: &str, output_path: &Path) -> Result<(), Vec<String>> {
    let ir = compile_source(source)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| vec![format!("IO error: {}", e)])?;
    }

    let ir_path = output_path.with_extension("ll");
    fs::write(&ir_path, &ir).map_err(|e| vec![format!("IO error: {}", e)])?;

    let status = process::Command::new("clang")
        .arg("-o")
        .arg(output_path)
        .arg(&ir_path)
        .status()
        .map_err(|e| vec![format!("Failed to run clang: {}", e)])?;

    let _ = fs::remove_file(&ir_path);

    if status.success() {
        Ok(())
    } else {
        Err(vec!["clang compilation failed. Is clang installed?".into()])
    }
}

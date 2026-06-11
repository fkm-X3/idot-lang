mod lexer;
mod parser;
mod codegen;

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let source = if args.len() > 1 {
        fs::read_to_string(&args[1]).unwrap_or_else(|e| {
            eprintln!("Error reading file: {}", e);
            process::exit(1);
        })
    } else {
        eprintln!("Usage: {} <source_file>", args[0]);
        process::exit(1);
    };

    let mut lexer = lexer::Lexer::new(&source);
    let tokens = lexer.tokenize();

    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program();

    if !parser.errors.is_empty() {
        for err in &parser.errors {
            eprintln!("Parse error: {}", err);
        }
        process::exit(1);
    }

    let mut codegen = codegen::CodeGen::new();
    let ir = match codegen.compile(&program) {
        Ok(ir) => ir,
        Err(e) => {
            eprintln!("Codegen error: {}", e);
            process::exit(1);
        }
    };

    let out_dir = env::temp_dir().join("idot");
    let _ = fs::create_dir_all(&out_dir);

    let ir_path = out_dir.join("out.ll");
    let exe_path = out_dir.join("out.exe");

    fs::write(&ir_path, &ir).unwrap_or_else(|e| {
        eprintln!("Error writing IR file: {}", e);
        process::exit(1);
    });

    println!("--- Generated LLVM IR ---");
    println!("{}", ir);
    println!("--- Compiling with clang ---");

    let clang_status = process::Command::new("clang")
        .arg("-o")
        .arg(&exe_path)
        .arg(&ir_path)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Error running clang: {}", e);
            process::exit(1);
        });

    if !clang_status.success() {
        eprintln!("clang compilation failed");
        process::exit(1);
    }

    println!("--- Running program ---");
    let run_status = process::Command::new(&exe_path)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Error running program: {}", e);
            process::exit(1);
        });

    println!("Program exited with code: {}", run_status.code().unwrap_or(-1));
}

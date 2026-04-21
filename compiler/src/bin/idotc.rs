use std::fs;

use idot::backend::c_backend;
use idot::{lexer, parser};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: idotc <input.idot> [output.c]");
        std::process::exit(1);
    }

    let input = match fs::read_to_string(&args[1]) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("Failed to read {}: {error}", args[1]);
            std::process::exit(1);
        }
    };

    let tokens = match lexer::scan_tokens(&input) {
        Ok(tokens) => tokens,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    let statements = match parser::parse(tokens) {
        Ok(statements) => statements,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    let c_output = match c_backend::emit_c(&statements) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    let output_path = if args.len() >= 3 { &args[2] } else { "a.out.c" };
    if let Err(error) = fs::write(output_path, c_output) {
        eprintln!("Failed to write {output_path}: {error}");
        std::process::exit(1);
    }

    println!("Wrote {output_path}");
}

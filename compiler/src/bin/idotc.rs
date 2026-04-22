use std::fs;

use idot::backend::native_backend;
use idot::{lexer, parser};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: idotc <input.idot>");
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

    if let Err(error) = native_backend::run_native(&statements) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

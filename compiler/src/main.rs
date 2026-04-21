use std::fs;
use std::io::{self, Write};

use idot::session::Session;

fn run_file(path: &str) -> i32 {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Failed to open file: {path} ({error})");
            return 1;
        }
    };

    let mut session = Session::new();
    if let Err(error) = session.execute(&source, &mut io::stdout()) {
        eprintln!("{error}");
        return 1;
    }

    0
}

fn run_repl() -> i32 {
    let mut session = Session::new();
    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        print!("idot> ");
        if io::stdout().flush().is_err() {
            eprintln!("Failed to flush stdout.");
            return 1;
        }

        line.clear();
        match stdin.read_line(&mut line) {
            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {}
            Err(error) => {
                eprintln!("Failed to read from stdin: {error}");
                return 1;
            }
        }

        let trimmed = line.trim();
        if trimmed == "exit" || trimmed == "quit" {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }

        if let Err(error) = session.execute(trimmed, &mut io::stdout()) {
            eprintln!("{error}");
        }
    }

    0
}

fn run(args: &[String]) -> i32 {
    if args.len() > 2 {
        eprintln!("Usage: idot [file.idot]");
        return 1;
    }

    if args.len() == 2 {
        return run_file(&args[1]);
    }

    run_repl()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    std::process::exit(run(&args));
}

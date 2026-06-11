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

    let out_dir = env::temp_dir().join("idot");
    let exe_path = out_dir.join("out.exe");

    match idot_lang::compile_to_exe(&source, &exe_path) {
        Ok(()) => {
            println!("--- Running program ---");
            let status = process::Command::new(&exe_path)
                .status()
                .unwrap_or_else(|e| {
                    eprintln!("Error running program: {}", e);
                    process::exit(1);
                });
            println!("Program exited with code: {}", status.code().unwrap_or(-1));
        }
        Err(errors) => {
            for e in &errors {
                eprintln!("Error: {}", e);
            }
            process::exit(1);
        }
    }
}

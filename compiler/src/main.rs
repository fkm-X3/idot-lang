use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: idot compile <file.ido> [--emit-c]");
        eprintln!("       idot run    <file.ido>");
        std::process::exit(1);
    }

    let command = &args[1];
    let file_path = &args[2];

    match command.as_str() {
        "compile" => {
            let emit_c = args.iter().any(|a| a == "--emit-c");
            do_compile(file_path, emit_c);
        }
        "run" => {
            do_run(file_path);
        }
        _ => {
            eprintln!("Unknown command: {}. Use 'compile' or 'run'.", command);
            std::process::exit(1);
        }
    }
}

fn do_compile(input_path: &str, emit_c: bool) {
    let source = fs::read_to_string(input_path)
        .unwrap_or_else(|e| {
            eprintln!("Error reading file '{}': {}", input_path, e);
            std::process::exit(1);
        });

    let c_source = idot::compile(&source);

    let stem = if let Some(s) = input_path.strip_suffix(".ido") {
        s
    } else {
        input_path
    };
    let c_path = format!("{}.c", stem);
    let exe_path = if cfg!(target_os = "windows") {
        format!("{}.exe", stem)
    } else {
        stem.to_string()
    };

    fs::write(&c_path, &c_source)
        .unwrap_or_else(|e| {
            eprintln!("Error writing C file '{}': {}", c_path, e);
            std::process::exit(1);
        });

    if emit_c {
        println!("Wrote C output to {}", c_path);
        return;
    }

    // Compile with cc
    let cc = if cfg!(target_os = "windows") { "clang" } else { "cc" };
    let status = Command::new(cc)
        .arg("-o")
        .arg(&exe_path)
        .arg(&c_path)
        .status()
        .unwrap_or_else(|_| {
            eprintln!("Failed to run '{}'. Is a C compiler installed?", cc);
            std::process::exit(1);
        });

    if !status.success() {
        eprintln!("C compilation failed");
        std::process::exit(1);
    }

    println!("Compiled to {}", exe_path);
}

fn do_run(input_path: &str) {
    let source = fs::read_to_string(input_path)
        .unwrap_or_else(|e| {
            eprintln!("Error reading file '{}': {}", input_path, e);
            std::process::exit(1);
        });

    let c_source = idot::compile(&source);

    let stem = if let Some(s) = input_path.strip_suffix(".ido") {
        s
    } else {
        input_path
    };
    let c_path = format!("{}.c", stem);
    let exe_path = if cfg!(target_os = "windows") {
        format!("{}.exe", stem)
    } else {
        stem.to_string()
    };

    fs::write(&c_path, &c_source).expect("Failed to write C file");

    let cc = if cfg!(target_os = "windows") { "clang" } else { "cc" };
    let status = Command::new(cc)
        .arg("-o")
        .arg(&exe_path)
        .arg(&c_path)
        .status()
        .expect("Failed to compile C output");

    if !status.success() {
        eprintln!("C compilation failed");
        std::process::exit(1);
    }

    let status = Command::new(&exe_path)
        .status()
        .expect("Failed to run executable");

    std::process::exit(status.code().unwrap_or(1));
}

use std::env;
use std::fs;
use std::path::Path;
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

fn do_compile(input_path_str: &str, emit_c: bool) {
    let input_path = Path::new(input_path_str);
    let source = fs::read_to_string(input_path)
        .unwrap_or_else(|e| {
            eprintln!("Error reading file '{}': {}", input_path.display(), e);
            std::process::exit(1);
        });

    let c_source = idot::compile_with_path(&source, Some(input_path));

    let stem = input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let build_dir = Path::new("build");
    fs::create_dir_all(build_dir).ok();
    let c_path = build_dir.join(format!("{}.c", stem));
    let exe_path = if cfg!(target_os = "windows") {
        build_dir.join(format!("{}.exe", stem))
    } else {
        build_dir.join(stem)
    };

    fs::write(&c_path, &c_source)
        .unwrap_or_else(|e| {
            eprintln!("Error writing C file '{}': {}", c_path.display(), e);
            std::process::exit(1);
        });

    if emit_c {
        eprintln!("Wrote C output to {}", c_path.display());
        return;
    }

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

    println!("Compiled to {}", exe_path.display());
}

fn do_run(input_path_str: &str) {
    let input_path = Path::new(input_path_str);
    let source = fs::read_to_string(input_path)
        .unwrap_or_else(|e| {
            eprintln!("Error reading file '{}': {}", input_path.display(), e);
            std::process::exit(1);
        });

    let c_source = idot::compile_with_path(&source, Some(input_path));

    let stem = input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let build_dir = Path::new("build");
    fs::create_dir_all(build_dir).ok();
    let c_path = build_dir.join(format!("{}.c", stem));
    let exe_path = if cfg!(target_os = "windows") {
        build_dir.join(format!("{}.exe", stem))
    } else {
        build_dir.join(stem)
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

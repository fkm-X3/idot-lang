use std::fs;

use idot::session::Session;

fn compile_file(path: &str) -> i32 {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Failed to open file: {path} ({error})");
            return 1;
        }
    };

    let obj_path = format!("{}.o", path);
    
    if let Err(error) = Session::compile_aot(&source, &obj_path) {
        eprintln!("{error}");
        return 1;
    }

    println!("Successfully compiled to {}", obj_path);
    0
}

fn run(args: &[String]) -> i32 {
    if args.len() != 2 {
        eprintln!("Usage: idot <file.idot>");
        eprintln!("Note: idot now compiles to object files (.o)");
        eprintln!("Use a linker like 'link.exe' or 'ld' to create executables");
        return 1;
    }

    compile_file(&args[1])
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    std::process::exit(run(&args));
}


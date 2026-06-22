mod manifest;
mod commands;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: matrix <command> [options]");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  new <project-name>   Create a new project");
        eprintln!("  build                Build the project in current directory");
        eprintln!("  run                  Build and run the project");
        eprintln!("  test                 Run tests");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "new" => {
            if args.len() < 3 {
                eprintln!("Usage: matrix new <project-name>");
                std::process::exit(1);
            }
            commands::cmd_new(&args[2]);
        }
        "build" => {
            let dir = std::env::current_dir().unwrap_or_else(|e| {
                eprintln!("Error getting current directory: {}", e);
                std::process::exit(1);
            });
            commands::cmd_build(&dir);
        }
        "run" => {
            let dir = std::env::current_dir().unwrap_or_else(|e| {
                eprintln!("Error getting current directory: {}", e);
                std::process::exit(1);
            });
            commands::cmd_run(&dir);
        }
        "test" => {
            let dir = std::env::current_dir().unwrap_or_else(|e| {
                eprintln!("Error getting current directory: {}", e);
                std::process::exit(1);
            });
            commands::cmd_test(&dir);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

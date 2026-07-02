mod manifest;
mod commands;
mod deps;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: matrix <command> [options]");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  new <project-name>   Create a new project");
        eprintln!("  build                Build the project in current directory");
        eprintln!("  run                  Build and run the project");
        eprintln!("  test [options]       Run tests (--self-hosted to use Idot compiler)");
        eprintln!("  add <name> [url]     Add a dependency");
        eprintln!("  remove <name>        Remove a dependency");
        eprintln!("  vendor               Download all dependencies locally");
        eprintln!("  init                 Initialize a project in the current directory");
        eprintln!("  update               Pull latest idot and rebuild the compiler");
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
            let dir = current_dir();
            commands::cmd_build(&dir);
        }
        "run" => {
            let dir = current_dir();
            commands::cmd_run(&dir);
        }
        "test" => {
            let dir = current_dir();
            let self_hosted = args[2..].iter().any(|a| a == "--self-hosted");
            commands::cmd_test(&dir, self_hosted);
        }
        "add" => {
            let dir = current_dir();
            commands::cmd_add(&dir, &args[2..]);
        }
        "remove" => {
            if args.len() < 3 {
                eprintln!("Usage: matrix remove <name>");
                std::process::exit(1);
            }
            let dir = current_dir();
            commands::cmd_remove(&dir, &args[2]);
        }
        "vendor" => {
            let dir = current_dir();
            commands::cmd_vendor(&dir);
        }
        "init" => {
            commands::cmd_init();
        }
        "update" => {
            commands::cmd_update();
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

fn current_dir() -> std::path::PathBuf {
    std::env::current_dir().unwrap_or_else(|e| {
        eprintln!("Error getting current directory: {}", e);
        std::process::exit(1);
    })
}

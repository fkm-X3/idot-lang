use std::fs;
use std::path::Path;
use crate::manifest::Manifest;

pub fn cmd_new(project_name: &str) {
    let dir = Path::new(project_name);
    if dir.exists() {
        eprintln!("Error: directory '{}' already exists", project_name);
        std::process::exit(1);
    }

    fs::create_dir_all(dir.join("src"))
        .unwrap_or_else(|e| {
            eprintln!("Error creating project directory: {}", e);
            std::process::exit(1);
        });

    // Write matrix.toml
    let manifest = Manifest::template(project_name);
    fs::write(dir.join("matrix.toml"), &manifest)
        .unwrap_or_else(|e| {
            eprintln!("Error writing matrix.toml: {}", e);
            std::process::exit(1);
        });

    // Write src/main.ido
    let main_src = format!(
        r#"fn main() -> i32 {{
    return 0;
}}
"#
    );
    fs::write(dir.join("src").join("main.ido"), &main_src)
        .unwrap_or_else(|e| {
            eprintln!("Error writing src/main.ido: {}", e);
            std::process::exit(1);
        });

    println!("Created project '{}'", project_name);
    println!("  {}/matrix.toml", project_name);
    println!("  {}/src/main.ido", project_name);
}

pub fn cmd_build(project_dir: &Path) {
    let manifest_path = project_dir.join("matrix.toml");
    if !manifest_path.exists() {
        eprintln!("Error: no matrix.toml found in '{}'", project_dir.display());
        std::process::exit(1);
    }

    let manifest = Manifest::load(&manifest_path)
        .unwrap_or_else(|e| {
            eprintln!("Error parsing matrix.toml: {}", e);
            std::process::exit(1);
        });

    let main_ido = project_dir.join("src").join("main.ido");
    if !main_ido.exists() {
        eprintln!("Error: no src/main.ido found in project");
        std::process::exit(1);
    }

    // Read and compile the Idot source
    let source = fs::read_to_string(&main_ido)
        .unwrap_or_else(|e| {
            eprintln!("Error reading {}: {}", main_ido.display(), e);
            std::process::exit(1);
        });

    let c_source = idot::compile(&source);

    let c_path = project_dir.join("build").join(format!("{}.c", manifest.name));
    let exe_name = if cfg!(target_os = "windows") {
        format!("{}.exe", manifest.name)
    } else {
        manifest.name.clone()
    };
    let exe_path = project_dir.join("build").join(&exe_name);

    fs::create_dir_all(project_dir.join("build"))
        .unwrap_or_else(|e| {
            eprintln!("Error creating build directory: {}", e);
            std::process::exit(1);
        });

    fs::write(&c_path, &c_source)
        .unwrap_or_else(|e| {
            eprintln!("Error writing C file: {}", e);
            std::process::exit(1);
        });

    let cc = if cfg!(target_os = "windows") { "clang" } else { "cc" };
    let status = std::process::Command::new(cc)
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

    println!("Compiled {} → {}", manifest.name, exe_path.display());
}

pub fn cmd_run(project_dir: &Path) {
    cmd_build(project_dir);

    let manifest_path = project_dir.join("matrix.toml");
    let manifest = Manifest::load(&manifest_path).expect("Failed to load manifest");

    let exe_name = if cfg!(target_os = "windows") {
        format!("{}.exe", manifest.name)
    } else {
        manifest.name.clone()
    };
    let exe_path = project_dir.join("build").join(&exe_name);

    let status = std::process::Command::new(&exe_path)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to run executable: {}", e);
            std::process::exit(1);
        });

    std::process::exit(status.code().unwrap_or(1));
}

pub fn cmd_test(project_dir: &Path) {
    // Find all *_test.ido files in src/
    let src_dir = project_dir.join("src");
    if !src_dir.exists() {
        eprintln!("Error: no src/ directory found");
        std::process::exit(1);
    }

    let test_files: Vec<_> = fs::read_dir(&src_dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "ido" {
                let name = path.file_stem()?.to_str()?;
                if name.ends_with("_test") {
                    return Some(path);
                }
            }
            None
        })
        .collect();

    if test_files.is_empty() {
        println!("No tests found");
        return;
    }

    for test_file in &test_files {
        println!("Testing {}...", test_file.display());
        let source = fs::read_to_string(test_file).expect("Failed to read test file");
        let c_source = idot::compile(&source);

        let c_path = project_dir.join("build").join(
            format!("{}_test.c", test_file.file_stem().unwrap().to_str().unwrap())
        );
        let exe_path = project_dir.join("build").join(
            format!("{}_test{}", test_file.file_stem().unwrap().to_str().unwrap(),
                if cfg!(target_os = "windows") { ".exe" } else { "" })
        );

        fs::create_dir_all(project_dir.join("build")).ok();
        fs::write(&c_path, &c_source).expect("Failed to write C file");

        let cc = if cfg!(target_os = "windows") { "clang" } else { "cc" };
        let status = std::process::Command::new(cc)
            .arg("-o")
            .arg(&exe_path)
            .arg(&c_path)
            .status()
            .expect("Failed to compile C output");

        if !status.success() {
            eprintln!("  FAILED to compile test");
            continue;
        }

        let status = std::process::Command::new(&exe_path)
            .status()
            .expect("Failed to run test");

        if status.success() {
            println!("  PASSED");
        } else {
            println!("  FAILED with exit code {:?}", status.code());
        }
    }
}

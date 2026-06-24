use std::fs;
use std::path::{Path, PathBuf};
use crate::manifest::Manifest;
use crate::deps;

fn get_stdlib_dir() -> PathBuf {
    let matrix_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    matrix_dir.parent().unwrap().join("lib")
}

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

    let manifest = Manifest::template(project_name);
    fs::write(dir.join("matrix.toml"), &manifest)
        .unwrap_or_else(|e| {
            eprintln!("Error writing matrix.toml: {}", e);
            std::process::exit(1);
        });

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
    let manifest = load_manifest(project_dir);
    let mut import_dirs = resolve_deps(project_dir, &manifest);
    import_dirs.push(get_stdlib_dir());

    let main_ido = project_dir.join("src").join("main.ido");
    if !main_ido.exists() {
        eprintln!("Error: no src/main.ido found in project");
        std::process::exit(1);
    }

    let source = fs::read_to_string(&main_ido)
        .unwrap_or_else(|e| {
            eprintln!("Error reading {}: {}", main_ido.display(), e);
            std::process::exit(1);
        });

    let c_source = idot::compile_with_deps(&source, Some(&main_ido), &import_dirs);

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

    let manifest = load_manifest(project_dir);

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
    let src_dir = project_dir.join("src");
    if !src_dir.exists() {
        eprintln!("Error: no src/ directory found");
        std::process::exit(1);
    }

    let manifest = load_manifest(project_dir);
    let mut import_dirs = resolve_deps(project_dir, &manifest);
    import_dirs.push(get_stdlib_dir());

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
        let c_source = idot::compile_with_deps(&source, Some(test_file), &import_dirs);

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

pub fn cmd_add(project_dir: &Path, args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: matrix add <name> [git_url] [tag]");
        std::process::exit(1);
    }

    let name = &args[0];
    let manifest_path = project_dir.join("matrix.toml");
    let mut manifest_content = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| {
            eprintln!("Error reading matrix.toml: {}", e);
            std::process::exit(1);
        });

    // Check if dependency already exists
    if manifest_content.contains(&format!("{} = ", name)) || manifest_content.contains(&format!("{}=", name)) {
        eprintln!("Dependency '{}' already exists in matrix.toml", name);
        std::process::exit(1);
    }

    let dep_line = if args.len() >= 2 {
        let url = &args[1];
        if args.len() >= 3 {
            let tag = &args[2];
            format!("{} = {{ git = \"{}\", tag = \"{}\" }}\n", name, url, tag)
        } else {
            format!("{} = {{ git = \"{}\" }}\n", name, url)
        }
    } else {
        format!("{} = {{}}\n", name)
    };

    // Insert before the last newline, or append at the end of [dependencies]
    if manifest_content.contains("[dependencies]") {
        // Find the [dependencies] section and add after it (or at end of section)
        let lines: Vec<&str> = manifest_content.lines().collect();
        let mut in_deps = false;
        let mut insert_pos = manifest_content.len();

        for (i, line) in lines.iter().enumerate() {
            if *line == "[dependencies]" {
                in_deps = true;
                insert_pos = manifest_content.len();
                continue;
            }
            if in_deps {
                if line.starts_with('[') {
                    insert_pos = lines[..i].join("\n").len() + 1;
                    break;
                }
                insert_pos = lines[..=i].join("\n").len() + 1;
            }
        }

        if in_deps {
            manifest_content.insert_str(insert_pos, &dep_line);
        } else {
            // No [dependencies] section - shouldn't happen with template, but handle gracefully
            manifest_content.push_str(&format!("\n[dependencies]\n{}", dep_line));
        }
    } else {
        manifest_content.push_str(&format!("\n[dependencies]\n{}", dep_line));
    }

    fs::write(&manifest_path, &manifest_content)
        .unwrap_or_else(|e| {
            eprintln!("Error writing matrix.toml: {}", e);
            std::process::exit(1);
        });

    println!("Added dependency '{}'", name);
}

pub fn cmd_remove(project_dir: &Path, name: &str) {
    let manifest_path = project_dir.join("matrix.toml");
    let manifest_content = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| {
            eprintln!("Error reading matrix.toml: {}", e);
            std::process::exit(1);
        });

    let lines: Vec<&str> = manifest_content.lines().collect();
    let mut new_lines = Vec::new();
    let mut removed = false;
    let mut in_deps = false;

    for line in &lines {
        if *line == "[dependencies]" {
            in_deps = true;
            new_lines.push(line.to_string());
            continue;
        }
        if in_deps {
            if line.starts_with('[') {
                in_deps = false;
                new_lines.push(line.to_string());
                continue;
            }
            if let Some((dep_name, _)) = line.split_once('=') {
                if dep_name.trim() == name {
                    removed = true;
                    continue;
                }
            }
        }
        new_lines.push(line.to_string());
    }

    if !removed {
        eprintln!("Dependency '{}' not found in matrix.toml", name);
        std::process::exit(1);
    }

    fs::write(&manifest_path, new_lines.join("\n"))
        .unwrap_or_else(|e| {
            eprintln!("Error writing matrix.toml: {}", e);
            std::process::exit(1);
        });

    println!("Removed dependency '{}'", name);
}

pub fn cmd_vendor(project_dir: &Path) {
    let manifest = load_manifest(project_dir);
    deps::vendor_all(&manifest, project_dir);
}

fn load_manifest(project_dir: &Path) -> Manifest {
    let manifest_path = project_dir.join("matrix.toml");
    if !manifest_path.exists() {
        eprintln!("Error: no matrix.toml found in '{}'", project_dir.display());
        std::process::exit(1);
    }

    Manifest::load(&manifest_path).unwrap_or_else(|e| {
        eprintln!("Error parsing matrix.toml: {}", e);
        std::process::exit(1);
    })
}

fn resolve_deps(project_dir: &Path, manifest: &Manifest) -> Vec<PathBuf> {
    if manifest.dependencies.is_empty() {
        return Vec::new();
    }

    println!("Resolving dependencies...");
    let dep_paths = deps::fetch_all(manifest, project_dir);
    let import_dirs = deps::resolve_import_paths(&dep_paths);
    if !import_dirs.is_empty() {
        println!("  import paths: {:?}", import_dirs);
    }
    import_dirs
}

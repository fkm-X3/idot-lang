use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use crate::manifest::{Manifest, DepEntry};

pub fn fetch_all(manifest: &Manifest, project_dir: &Path) -> Vec<PathBuf> {
    let deps_dir = project_dir.join("deps");
    let mut dep_paths = Vec::new();

    for (name, entry) in &manifest.dependencies {
        let target = deps_dir.join(name);
        if let Err(e) = fetch_dep(name, entry, &target) {
            eprintln!("Warning: failed to fetch dependency '{}': {}", name, e);
        } else {
            dep_paths.push(target);
        }
    }

    dep_paths
}

pub fn vendor_all(manifest: &Manifest, project_dir: &Path) {
    let vendor_dir = project_dir.join("vendor");
    fs::create_dir_all(&vendor_dir).unwrap_or_else(|e| {
        eprintln!("Error creating vendor directory: {}", e);
        std::process::exit(1);
    });

    for (name, entry) in &manifest.dependencies {
        let target = vendor_dir.join(name);
        println!("Vendoring {}...", name);
        if let Err(e) = fetch_dep(name, entry, &target) {
            eprintln!("Error vendoring '{}': {}", name, e);
        }
    }

    println!("All dependencies vendored to 'vendor/'");
}

fn fetch_dep(name: &str, entry: &DepEntry, target: &Path) -> Result<(), String> {
    let git_url = match &entry.git {
        Some(url) => url,
        None => return Err("no git URL specified".to_string()),
    };

    if target.exists() {
        let status = std::process::Command::new("git")
            .args(["-C", &target.to_string_lossy(), "fetch", "--tags"])
            .status()
            .map_err(|e| format!("failed to run git: {}", e))?;

        if !status.success() {
            return Err("git fetch failed".to_string());
        }

        if let Some(tag) = &entry.tag {
            let status = std::process::Command::new("git")
                .args(["-C", &target.to_string_lossy(), "checkout", &format!("tags/{}", tag), "-q"])
                .status()
                .map_err(|e| format!("failed to checkout tag: {}", e))?;
            if !status.success() {
                return Err(format!("failed to checkout tag '{}'", tag));
            }
        } else if let Some(branch) = &entry.branch {
            let status = std::process::Command::new("git")
                .args(["-C", &target.to_string_lossy(), "checkout", branch, "-q"])
                .status()
                .map_err(|e| format!("failed to checkout branch: {}", e))?;
            if !status.success() {
                return Err(format!("failed to checkout branch '{}'", branch));
            }
        }

        return Ok(());
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create dir: {}", e))?;
    }

    let branch_or_tag = entry.tag.as_deref().or(entry.branch.as_deref());
    let target_str = target.to_string_lossy().into_owned();

    let mut cmd = std::process::Command::new("git");
    cmd.arg("clone").arg("--depth").arg("1");
    if let Some(bt) = branch_or_tag {
        cmd.arg("--branch").arg(bt);
    }
    cmd.arg(git_url).arg(&target_str);

    let status = cmd.status()
        .map_err(|e| format!("failed to run git: {}", e))?;

    if !status.success() {
        return Err(format!("git clone failed for '{}'", name));
    }

    Ok(())
}

pub fn resolve_import_paths(dep_paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();

    for dep_dir in dep_paths {
        let src_dir = dep_dir.join("src");
        if src_dir.exists() {
            if seen.insert(src_dir.clone()) {
                paths.push(src_dir);
            }
        } else if seen.insert(dep_dir.to_path_buf()) {
            paths.push(dep_dir.to_path_buf());
        }
    }

    paths
}

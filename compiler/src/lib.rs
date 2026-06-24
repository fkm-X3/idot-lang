pub mod ast;
pub mod lexer;
pub mod parser;
pub mod codegen;
pub mod semantic;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::fs;
use codegen::c::CBackend;
use parser::Parser;
use semantic::SemanticAnalyzer;

pub fn compile(source: &str) -> String {
    compile_with_path(source, None)
}

pub fn compile_with_path(source: &str, file_path: Option<&Path>) -> String {
    compile_with_deps(source, file_path, &[])
}

pub fn compile_with_deps(
    source: &str,
    file_path: Option<&Path>,
    include_dirs: &[PathBuf],
) -> String {
    let base_dir = file_path.and_then(|p| p.parent()).map(|p| p.to_path_buf());

    let mut program = parse_with_imports(source, base_dir.as_deref(), &mut HashSet::new(), include_dirs);

    let mut analyzer = SemanticAnalyzer::new();
    if let Err(errors) = analyzer.analyze(&mut program) {
        for err in &errors {
            eprintln!("semantic error: {}", err);
        }
        eprintln!("Compilation aborted due to {} semantic error(s)", errors.len());
        std::process::exit(1);
    }

    // Insert monomorphized function definitions into the program
    program.extend(analyzer.monomorphized_fns.drain(..).map(ast::Decl::Fn));

    // Remove generic template functions — they were monomorphized into concrete versions
    program.retain(|decl| match decl {
        ast::Decl::Fn(f) => f.generic_params.is_empty(),
        _ => true,
    });

    let mut backend = CBackend::new();
    backend.generate(&program)
}

fn parse_with_imports(
    source: &str,
    base_dir: Option<&Path>,
    visited: &mut HashSet<PathBuf>,
    include_dirs: &[PathBuf],
) -> Vec<ast::Decl> {
    let mut parser = Parser::new(source);
    let program = parser.parse();

    // Resolve imports
    let mut resolved_decls: Vec<ast::Decl> = Vec::new();
    let mut imports_to_resolve: Vec<String> = Vec::new();

    // Extract imports before processing other declarations
    for decl in &program {
        if let ast::Decl::Import(i) = decl {
            imports_to_resolve.push(i.path.clone());
        }
    }

    // Resolve each import
    for import_path in &imports_to_resolve {
        let resolved = resolve_import_path(import_path, base_dir, include_dirs);
        match resolved {
            Some(resolved_path) => {
                if visited.contains(&resolved_path) {
                    continue;
                }
                visited.insert(resolved_path.clone());

                match fs::read_to_string(&resolved_path) {
                    Ok(import_src) => {
                        let import_decls = parse_with_imports(
                            &import_src,
                            resolved_path.parent(),
                            visited,
                            include_dirs,
                        );
                        resolved_decls.extend(import_decls);
                    }
                    Err(e) => {
                        eprintln!("Error reading import '{}': {}", resolved_path.display(), e);
                        std::process::exit(1);
                    }
                }
            }
            None => {
                eprintln!("Import not found: '{}'", import_path);
                std::process::exit(1);
            }
        }
    }

    // Add resolved imports first, then the rest of the declarations
    // (filtering out Import declarations themselves)
    let mut result: Vec<ast::Decl> = resolved_decls;
    for decl in program {
        if !matches!(decl, ast::Decl::Import(_)) {
            result.push(decl);
        }
    }

    result
}

fn resolve_import_path(import_path: &str, base_dir: Option<&Path>, include_dirs: &[PathBuf]) -> Option<PathBuf> {
    // First try relative to base_dir
    if let Some(base) = base_dir {
        let candidate = normalize_path(base.join(import_path));
        if candidate.exists() {
            return Some(candidate);
        }
    }

    // Then try each include directory
    for include_dir in include_dirs {
        let candidate = normalize_path(include_dir.join(import_path));
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let path = if path.extension().is_none() {
        path.with_extension("ido")
    } else {
        path
    };
    path
}


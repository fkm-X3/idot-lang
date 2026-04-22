use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn executes_program_with_native_backend() {
    let source =
        "let x = 3; if (x > 2) { print \"big\"; } else { print \"small\"; }\nprint x + 1;\n";
    let output = run_idotc(source);
    assert!(output.status.success());
    assert_eq!(normalized(stdout_text(output)), "big\n4\n");
}

#[test]
fn rejects_missing_input_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_idotc"))
        .output()
        .expect("idotc should execute");
    assert!(!output.status.success());
    assert!(stderr_text(output).contains("Usage: idotc <input.idot>"));
}

fn run_idotc(source: &str) -> std::process::Output {
    let path = write_temp_program(source);
    let output = Command::new(env!("CARGO_BIN_EXE_idotc"))
        .arg(&path)
        .output()
        .expect("idotc should execute");
    let _ = fs::remove_file(path);
    output
}

fn write_temp_program(source: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    path.push(format!("idot_native_test_{nanos}.idot"));
    fs::write(&path, source).expect("failed to create test source file");
    path
}

fn stdout_text(output: std::process::Output) -> String {
    String::from_utf8(output.stdout).expect("stdout should be valid UTF-8")
}

fn stderr_text(output: std::process::Output) -> String {
    String::from_utf8(output.stderr).expect("stderr should be valid UTF-8")
}

fn normalized(text: String) -> String {
    text.replace("\r\n", "\n")
}

mod lexer;
mod backend;

use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: idotc <input.idot> [output.c]");
        std::process::exit(1);
    }
    let input = fs::read_to_string(&args[1])?;
    let stmts = lexer::tokenize_and_parse(&input)?;
    let c = backend::c_backend::emit_c(&stmts)?;
    let out = if args.len() >= 3 { &args[2] } else { "a.out.c" };
    fs::write(out, c)?;
    println!("Wrote {}", out);
    Ok(())
}

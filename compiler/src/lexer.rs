#[derive(Debug)]
pub enum Stmt {
    PrintString(String),
    PrintNumber(i64),
}

pub fn tokenize_and_parse(src: &str) -> Result<Vec<Stmt>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    for raw_line in src.lines() {
        let mut line = raw_line.trim();
        if line.is_empty() { continue; }
        if line.ends_with(';') {
            line = line.trim_end_matches(';').trim();
        }
        if let Some(rest) = line.strip_prefix("print ") {
            let rest = rest.trim();
            if rest.starts_with('"') && rest.ends_with('"') && rest.len() >= 2 {
                let s = &rest[1..rest.len()-1];
                out.push(Stmt::PrintString(s.to_string()));
            } else if let Ok(n) = rest.parse::<i64>() {
                out.push(Stmt::PrintNumber(n));
            } else {
                return Err(format!("Unsupported print argument: {}", rest).into());
            }
        } else {
            return Err(format!("Unsupported statement: {}", line).into());
        }
    }
    Ok(out)
}

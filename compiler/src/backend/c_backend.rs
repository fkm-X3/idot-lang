fn escape_c_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

pub fn emit_c(stmts: &Vec<crate::lexer::Stmt>) -> Result<String, Box<dyn std::error::Error>> {
    let mut out = String::new();
    out.push_str("#include <stdio.h>\n#include <stdlib.h>\n\nint main(void) {\n");
    for stmt in stmts {
        match stmt {
            crate::lexer::Stmt::PrintString(s) => {
                let esc = escape_c_string(s);
                out.push_str(&format!("    printf(\"%s\\n\", \"{}\");\n", esc));
            },
            crate::lexer::Stmt::PrintNumber(n) => {
                out.push_str(&format!("    printf(\"%lld\\n\", (long long){});\n", n));
            },
        }
    }
    out.push_str("    return 0;\n}\n");
    Ok(out)
}

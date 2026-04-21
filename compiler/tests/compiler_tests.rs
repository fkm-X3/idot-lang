use idot::backend::c_backend;
use idot::{lexer, parser};

#[test]
fn emits_c_program() {
    let source = "let x = 3; if (x > 2) { print \"big\"; } else { print \"small\"; }";
    let tokens = lexer::scan_tokens(source).expect("lexer should succeed");
    let statements = parser::parse(tokens).expect("parser should succeed");
    let generated = c_backend::emit_c(&statements).expect("C backend should succeed");

    assert!(generated.contains("int main(void)"));
    assert!(generated.contains("idot_print"));
}

use idot::diagnostics::ErrorPhase;
use idot::session::Session;

#[test]
fn conditionals_and_variables() {
    let mut session = Session::new();
    let mut output = Vec::new();
    session
        .execute(
            "let x = 3;\nif (x > 2) { print \"big\"; } else { print \"small\"; }\nprint x + 1;\n",
            &mut output,
        )
        .expect("program should execute");

    assert_eq!(String::from_utf8(output).unwrap(), "big\n4\n");
}

#[test]
fn block_scope() {
    let mut session = Session::new();
    let mut output = Vec::new();
    session
        .execute(
            "let x = 1;\n{\n  let x = 2;\n  print x;\n}\nprint x;\n",
            &mut output,
        )
        .expect("program should execute");

    assert_eq!(String::from_utf8(output).unwrap(), "2\n1\n");
}

#[test]
fn runtime_error_for_undefined_variable() {
    let mut session = Session::new();
    let mut output = Vec::new();
    let error = session
        .execute("print missing;\n", &mut output)
        .expect_err("expected runtime error");

    assert_eq!(error.phase(), ErrorPhase::Runtime);
}

#[test]
fn parse_error_for_invalid_declaration() {
    let mut session = Session::new();
    let mut output = Vec::new();
    let error = session
        .execute("let x = ;\n", &mut output)
        .expect_err("expected parse error");

    assert_eq!(error.phase(), ErrorPhase::Parse);
}

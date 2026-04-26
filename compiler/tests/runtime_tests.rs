use idot::diagnostics::ErrorPhase;
use idot::session;

#[test]
fn conditionals_and_variables() {
    let output = session::execute_source_to_string(
        "let x = 3;\nif (x > 2) { print \"big\"; } else { print \"small\"; }\nprint x + 1;\n",
    )
    .expect("program should execute");

    assert_eq!(output, "big\n4\n");
}

#[test]
fn block_scope() {
    let output = session::execute_source_to_string(
        "let x = 1;\n{\n  let x = 2;\n  print x;\n}\nprint x;\n",
    )
    .expect("program should execute");

    assert_eq!(output, "2\n1\n");
}

#[test]
fn runtime_error_for_undefined_variable() {
    let error = session::execute_source_to_string("print missing;\n")
        .expect_err("expected runtime error");

    assert_eq!(error.phase(), ErrorPhase::Runtime);
}

#[test]
fn parse_error_for_invalid_declaration() {
    let error = session::execute_source_to_string("let x = ;\n")
        .expect_err("expected parse error");

    assert_eq!(error.phase(), ErrorPhase::Parse);
}


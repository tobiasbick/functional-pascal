use super::*;

#[test]
fn unknown_name_has_correct_code_location_help() {
    use fpas_diagnostics::codes::SEMA_UNKNOWN_NAME;
    let errs = check_errors(
        "\
program T;
begin
  Foo()
end.",
    );
    let e = errs
        .iter()
        .find(|e| e.code == SEMA_UNKNOWN_NAME)
        .expect("expected SEMA_UNKNOWN_NAME");
    assert_eq!(e.span.line, 3, "wrong line");
    assert!(
        e.help.as_deref().is_some_and(|h| !h.is_empty()),
        "help text must be present"
    );
}

#[test]
fn duplicate_declaration_has_correct_code() {
    use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
    let errs = check_errors(
        "\
program T;
const X: integer := 1;
const X: integer := 2;
begin
end.",
    );
    let e = errs
        .iter()
        .find(|e| e.code == SEMA_DUPLICATE_DECLARATION)
        .expect("expected SEMA_DUPLICATE_DECLARATION");
    assert!(
        e.help.as_deref().is_some_and(|h| !h.is_empty()),
        "help text must be present"
    );
}

#[test]
fn type_mismatch_has_correct_code() {
    use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
    let errs = check_errors(
        "\
program T;
begin
  var N: integer := 'hello'
end.",
    );
    assert!(
        errs.iter().any(|e| e.code == SEMA_TYPE_MISMATCH),
        "expected SEMA_TYPE_MISMATCH; got: {errs:#?}"
    );
}

use super::*;

#[test]
fn division_by_zero_preserves_code_message_and_source_location() {
    let source = "\
program RegisterDivisionError;
begin
  var X: integer := 7 div 0
end.";
    let error = run_program(source).expect_err("division should fail");
    assert_eq!(
        error.code,
        fpas_diagnostics::codes::RUNTIME_DIVISION_BY_ZERO
    );
    assert!(
        error
            .message
            .to_ascii_lowercase()
            .contains("division by zero")
    );
    assert_eq!((error.span.line(), error.span.column()), (3, 21));
}

#[test]
fn explicit_panic_preserves_diagnostic_contract() {
    let source = "\
program RegisterPanic;
begin
  panic('expected failure')
end.";
    let error = run_program(source).expect_err("panic should fail");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
    assert!(error.message.contains("expected failure"));
    assert_eq!((error.span.line(), error.span.column()), (3, 3));
}

use super::*;

#[test]
fn intrinsic_arity_error_has_correct_code() {
    use fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT;
    let e = compile_err(
        "\
program T;
uses Std.Console;
begin
  var X: string := Std.Console.ReadLn(1)
end.",
    );
    assert_eq!(e.code, SEMA_WRONG_ARGUMENT_COUNT, "wrong diagnostic code");
    assert!(
        e.help.as_deref().is_some_and(|h| !h.is_empty()),
        "help text must be present"
    );
}

#[test]
fn runtime_vm_division_by_zero_has_code_column_and_help() {
    use fpas_diagnostics::codes::RUNTIME_DIVISION_BY_ZERO;

    let err = compile_run_error(
        "\
  program T;
  begin
    var X: integer := 10 div 0
  end.",
    );

    assert_eq!(
        err.code, RUNTIME_DIVISION_BY_ZERO,
        "wrong runtime diagnostic code"
    );
    assert_eq!(err.span.line, 3, "unexpected runtime error line");
    assert!(
        err.span.column > 0,
        "runtime column must be 1-based and non-zero"
    );
    assert!(
        err.help.as_deref().is_some_and(|h| !h.trim().is_empty()),
        "runtime help text must be present"
    );
}

#[test]
fn runtime_std_conversion_failure_has_code_column_and_help() {
    use fpas_diagnostics::codes::RUNTIME_CONVERSION_FAILURE;

    let err = compile_run_error(
        "\
  program T;
  begin
    var N: integer := Std.Conv.StrToInt('x')
  end.",
    );

    assert_eq!(
        err.code, RUNTIME_CONVERSION_FAILURE,
        "wrong std runtime diagnostic code"
    );
    assert_eq!(err.span.line, 3, "unexpected std runtime error line");
    assert!(
        err.span.column > 0,
        "std runtime column must be 1-based and non-zero"
    );
    assert!(
        err.help.as_deref().is_some_and(|h| !h.trim().is_empty()),
        "std runtime help text must be present"
    );
}

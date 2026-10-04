use super::{check_errors, check_ok};

#[test]
fn proc_run_accepts_string_command_and_string_array_args() {
    check_ok(
        r#"program T;
uses Std.Proc as Proc;
begin
  const Status: Result of (integer, string) := Proc.Run('tool', ['--help']);
  const Qualified: Result of (integer, string) := Proc.Run('tool', ['--version']);
end program;"#,
    );
}

#[test]
fn proc_capture_api_exposes_current_executable_and_process_output() {
    check_ok(
        r#"program T;
uses Std.Proc as Proc;
begin
  const Executable: Result of (string, string) := Proc.CurrentExecutable();
  const Captured: Result of (Proc.ProcessOutput, string) :=
    Proc.RunCapture('tool', ['--version']);
end program;"#,
    );
}

#[test]
fn proc_run_rejects_non_string_args_array() {
    let errs = check_errors(
        r#"program T;
uses Std.Proc as Proc;
begin
    const Status: Result of (integer, string) := Proc.Run('tool', [1, 2, 3]);
end program;"#,
    );

    assert_invalid_argument_elements(&errs);
}

#[test]
fn proc_run_capture_rejects_non_string_args_array() {
    let errs = check_errors(
        r#"program T;
uses Std.Proc as Proc;
begin
    const Captured: Result of (Proc.ProcessOutput, string) := Proc.RunCapture('tool', [1, 2, 3]);
end program;"#,
    );

    assert_invalid_argument_elements(&errs);
}

fn assert_invalid_argument_elements(errors: &[fpas_diagnostics::Diagnostic]) {
    let invalid_elements = errors
        .iter()
        .filter(|error| {
            error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH
                && error.message
                    == "Type mismatch in array element: expected `string`, found `integer`"
                && error
                    .span
                    .is_some_and(|span| span.length() == 1 && span.line() == 4)
        })
        .count();
    assert_eq!(invalid_elements, 3, "{errors:#?}");
}

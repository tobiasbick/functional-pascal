use super::{check_errors, check_ok};

#[test]
fn proc_run_accepts_string_command_and_string_array_args() {
    check_ok(
        r#"program T;
uses Std.Proc as Proc;
begin
  var Status: Result of integer, string := Proc.Run('tool', ['--help']);
  var Qualified: Result of integer, string := Proc.Run('tool', ['--version']);
end program;"#,
    );
}

#[test]
fn proc_capture_api_exposes_current_executable_and_process_output() {
    check_ok(
        r#"program T;
uses Std.Proc as Proc;
begin
  var Executable: Result of string, string := Proc.CurrentExecutable();
  var Captured: Result of Proc.ProcessOutput, string :=
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
    var Status: Result of integer, string := Proc.Run('tool', [1, 2, 3]);
end program;"#,
    );

    assert!(
        errs.iter().any(|e| e.message.contains("array of string")),
        "{errs:#?}"
    );
}

#[test]
fn proc_run_capture_rejects_non_string_args_array() {
    let errs = check_errors(
        r#"program T;
uses Std.Proc as Proc;
begin
    var Captured: Result of Proc.ProcessOutput, string := Proc.RunCapture('tool', [1, 2, 3]);
end program;"#,
    );

    assert!(
        errs.iter().any(|e| e.message.contains("array of string")),
        "{errs:#?}"
    );
}

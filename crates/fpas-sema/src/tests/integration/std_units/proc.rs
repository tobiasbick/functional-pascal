use super::{check_errors, check_ok};

#[test]
fn proc_run_accepts_string_command_and_string_array_args() {
    check_ok(
        "\
program T;
uses Std.Proc;
begin
  var Status: Result of integer, string := Run('tool', ['--help']);
  var Qualified: Result of integer, string := Std.Proc.Run('tool', ['--version'])
end.",
    );
}

#[test]
fn proc_capture_api_exposes_current_executable_and_process_output() {
    check_ok(
        "\
program T;
uses Std.Proc;
begin
  var Executable: Result of string, string := CurrentExecutable();
  var Captured: Result of ProcessOutput, string :=
    Std.Proc.RunCapture('tool', ['--version'])
end.",
    );
}

#[test]
fn proc_run_rejects_non_string_args_array() {
    let errs = check_errors(
        "\
program T;
uses Std.Proc;
begin
    var Status: Result of integer, string := Run('tool', [1, 2, 3])
end.",
    );

    assert!(
        errs.iter().any(|e| e.message.contains("array of string")),
        "{errs:#?}"
    );
}

#[test]
fn proc_run_capture_rejects_non_string_args_array() {
    let errs = check_errors(
        "\
program T;
uses Std.Proc;
begin
    var Captured: Result of ProcessOutput, string := RunCapture('tool', [1, 2, 3])
end.",
    );

    assert!(
        errs.iter().any(|e| e.message.contains("array of string")),
        "{errs:#?}"
    );
}

use super::super::super::support;

// ---------------------------------------------------------------------------
// Pop
// ---------------------------------------------------------------------------

#[test]
fn pop_returns_last() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  mutable var A: array of integer := [1, 2, 3];
  var Last: integer := Pop(A);
  WriteLn(Last);
  WriteLn(Length(A))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n2\n");
}

#[test]
fn pop_empty_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  mutable var A: array of integer := [];
  var X: integer := Pop(A)
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

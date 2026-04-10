use super::super::super::support;

// ---------------------------------------------------------------------------
// Push
// ---------------------------------------------------------------------------

#[test]
fn push_appends() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  mutable var A: array of integer := [1, 2];
  Push(A, 3);
  WriteLn(Length(A));
  WriteLn(A[2])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n3\n");
}

#[test]
fn push_to_empty() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  mutable var A: array of integer := [];
  Push(A, 42);
  WriteLn(Length(A));
  WriteLn(A[0])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n42\n");
}

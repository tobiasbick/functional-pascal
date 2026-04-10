use super::super::super::support;

// ---------------------------------------------------------------------------
// Contains
// ---------------------------------------------------------------------------

#[test]
fn contains_found() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [1, 2, 3];
  WriteLn(Contains(A, 2))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn contains_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [1, 2, 3];
  WriteLn(Contains(A, 99))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

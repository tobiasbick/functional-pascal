use super::super::super::support;

// ---------------------------------------------------------------------------
// Fill
// ---------------------------------------------------------------------------

#[test]
fn fill_normal() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := Fill(7, 3);
  WriteLn(Length(A));
  WriteLn(A[0]);
  WriteLn(A[2])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n7\n7\n");
}

#[test]
fn fill_zero_count() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := Fill(7, 0);
  WriteLn(Length(A))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn fill_string_elements() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of string := Fill('x', 2);
  WriteLn(A[0]);
  WriteLn(A[1])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "x\nx\n");
}

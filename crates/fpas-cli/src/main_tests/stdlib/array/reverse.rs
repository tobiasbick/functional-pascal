use super::super::super::support;

// ---------------------------------------------------------------------------
// Reverse
// ---------------------------------------------------------------------------

#[test]
fn reverse_normal() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [1, 2, 3];
  var R: array of integer := Reverse(A);
  WriteLn(R[0]);
  WriteLn(R[1]);
  WriteLn(R[2])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n2\n1\n");
}

#[test]
fn reverse_single_element() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [42];
  var R: array of integer := Reverse(A);
  WriteLn(R[0])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "42\n");
}

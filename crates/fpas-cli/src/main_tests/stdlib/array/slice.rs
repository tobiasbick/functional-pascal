use super::super::super::support;

// ---------------------------------------------------------------------------
// Slice
// ---------------------------------------------------------------------------

#[test]
fn slice_normal() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [10, 20, 30, 40];
  var C: array of integer := Slice(A, 1, 2);
  WriteLn(Length(C));
  WriteLn(C[0]);
  WriteLn(C[1])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n20\n30\n");
}

#[test]
fn slice_out_of_bounds_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [1, 2];
  var C: array of integer := Slice(A, 0, 10)
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

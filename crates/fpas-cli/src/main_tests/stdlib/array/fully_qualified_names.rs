use super::super::super::support;

// ---------------------------------------------------------------------------
// Fully qualified names
// ---------------------------------------------------------------------------

#[test]
fn fully_qualified_length() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [1, 2, 3];
  WriteLn(Std.Array.Length(A))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n");
}

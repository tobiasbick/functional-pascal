use super::super::super::support;

// ---------------------------------------------------------------------------
// Tan
// ---------------------------------------------------------------------------

#[test]
fn tan_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Tan(0.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

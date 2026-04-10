use super::super::super::support;

// ---------------------------------------------------------------------------
// Sin / Cos
// ---------------------------------------------------------------------------

#[test]
fn sin_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sin(0.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn cos_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Cos(0.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

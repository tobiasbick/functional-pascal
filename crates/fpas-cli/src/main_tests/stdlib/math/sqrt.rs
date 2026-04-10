use super::super::super::support;

// ---------------------------------------------------------------------------
// Sqrt
// ---------------------------------------------------------------------------

#[test]
fn sqrt_normal() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sqrt(16.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "4\n");
}

#[test]
fn sqrt_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sqrt(0.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn sqrt_negative_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sqrt(-1.0))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

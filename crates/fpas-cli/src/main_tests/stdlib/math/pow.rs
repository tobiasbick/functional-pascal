use super::super::super::support;

// ---------------------------------------------------------------------------
// Pow
// ---------------------------------------------------------------------------

#[test]
fn pow_normal() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Pow(2.0, 3.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "8\n");
}

#[test]
fn pow_zero_exponent() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Pow(5.0, 0.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

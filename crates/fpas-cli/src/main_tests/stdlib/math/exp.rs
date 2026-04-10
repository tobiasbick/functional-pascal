use super::super::super::support;

// ---------------------------------------------------------------------------
// Exp
// ---------------------------------------------------------------------------

#[test]
fn exp_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Exp(0.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn exp_one() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Round(Exp(1.0) * 1000.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    // e * 1000 ≈ 2718
    assert_eq!(stdout, "2718\n");
}

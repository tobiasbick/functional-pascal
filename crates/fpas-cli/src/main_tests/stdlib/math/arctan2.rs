use super::super::super::support;

// ---------------------------------------------------------------------------
// ArcTan2
// ---------------------------------------------------------------------------

#[test]
fn arctan2_quarter_pi() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Round(ArcTan2(1.0, 1.0) * 1000.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    // Pi/4 * 1000 ≈ 785
    assert_eq!(stdout, "785\n");
}

#[test]
fn arctan2_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(ArcTan2(0.0, 1.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

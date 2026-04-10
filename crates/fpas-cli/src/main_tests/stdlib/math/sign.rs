use super::super::super::support;

// ---------------------------------------------------------------------------
// Sign
// ---------------------------------------------------------------------------

#[test]
fn sign_positive_integer() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sign(42))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn sign_negative_integer() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sign(-7))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

#[test]
fn sign_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sign(0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn sign_real() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sign(-3.14))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

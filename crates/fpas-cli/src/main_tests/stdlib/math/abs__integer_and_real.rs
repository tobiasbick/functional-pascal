use super::super::super::support;

// ---------------------------------------------------------------------------
// Abs (integer and real)
// ---------------------------------------------------------------------------

#[test]
fn abs_negative_integer() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Abs(-7))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "7\n");
}

#[test]
fn abs_positive_integer() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Abs(5))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "5\n");
}

#[test]
fn abs_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Abs(0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn abs_negative_real() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Abs(-1.5))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert!(stdout.contains("1.5"), "got: {stdout}");
}

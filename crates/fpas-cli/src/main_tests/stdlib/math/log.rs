use super::super::super::support;

// ---------------------------------------------------------------------------
// Log
// ---------------------------------------------------------------------------

#[test]
fn log_e() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Round(Log(2.718281828459045)))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn log_one() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log(1.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn log_zero_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log(0.0))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

#[test]
fn log_negative_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log(-1.0))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

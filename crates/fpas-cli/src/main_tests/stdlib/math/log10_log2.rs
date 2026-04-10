use super::super::super::support;

// ---------------------------------------------------------------------------
// Log10 / Log2
// ---------------------------------------------------------------------------

#[test]
fn log10_hundred() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log10(100.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n");
}

#[test]
fn log10_one() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log10(1.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn log10_non_positive_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log10(0.0))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

#[test]
fn log2_eight() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log2(8.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n");
}

#[test]
fn log2_non_positive_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log2(-1.0))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

use super::super::super::support;

// ---------------------------------------------------------------------------
// Pi constant
// ---------------------------------------------------------------------------

#[test]
fn pi_constant() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Pi)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert!(stdout.starts_with("3.14"), "got: {stdout}");
}

#[test]
fn pi_fully_qualified() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Std.Math.Pi)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert!(stdout.starts_with("3.14"), "got: {stdout}");
}

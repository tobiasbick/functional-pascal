use super::super::super::support;

// ---------------------------------------------------------------------------
// Random / RandomInt / Randomize
// ---------------------------------------------------------------------------

#[test]
fn random_returns_value() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  var R: real := Random();
  WriteLn(R >= 0.0);
  WriteLn(R < 1.0)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\ntrue\n");
}

#[test]
fn random_int_returns_in_range() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  var N: integer := RandomInt(1, 1);
  WriteLn(N)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn random_int_reversed_bounds_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(RandomInt(2, 1))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

#[test]
fn randomize_no_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  Randomize();
  WriteLn('ok')
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "ok\n");
}

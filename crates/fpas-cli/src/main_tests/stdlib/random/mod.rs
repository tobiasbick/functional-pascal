use super::super::support;

#[test]
fn random_returns_value_from_std_random() {
    let source = r#"program T;
uses Std.Console, Std.Random;
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
fn random_int_returns_in_range_from_std_random() {
    let source = r#"program T;
uses Std.Console, Std.Random;
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
fn qualified_random_int_uses_std_random() {
    let source = r#"program T;
uses Std.Console, Std.Random;
begin
  WriteLn(Std.Random.RandomInt(4, 4))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "4\n");
}

#[test]
fn random_int_reversed_bounds_error() {
    let source = r#"program T;
uses Std.Console, Std.Random;
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
uses Std.Console, Std.Random;
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

#[test]
fn std_math_no_longer_imports_random_helpers() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(RandomInt(1, 1))
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(stderr.contains("Std.Random"), "stderr: {stderr}");
}

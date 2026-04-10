use super::super::super::support;

// ---------------------------------------------------------------------------
// Min / Max
// ---------------------------------------------------------------------------

#[test]
fn min_integer() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Min(3, 9))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n");
}

#[test]
fn max_integer() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Max(3, 9))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "9\n");
}

#[test]
fn min_equal_values() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Min(5, 5))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "5\n");
}

#[test]
fn max_equal_values() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Max(5, 5))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "5\n");
}

#[test]
fn min_negative_values() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Min(-10, -3))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-10\n");
}

#[test]
fn max_negative_values() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Max(-10, -3))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-3\n");
}

use super::super::super::support;

// ---------------------------------------------------------------------------
// Floor / Ceil / Round
// ---------------------------------------------------------------------------

#[test]
fn floor_positive() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Floor(2.9))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n");
}

#[test]
fn floor_negative() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Floor(-2.1))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-3\n");
}

#[test]
fn ceil_positive() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Ceil(2.1))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n");
}

#[test]
fn ceil_negative() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Ceil(-2.9))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-2\n");
}

#[test]
fn round_normal() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Round(2.6))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n");
}

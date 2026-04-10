use super::super::super::support;

// ---------------------------------------------------------------------------
// StartsWith / EndsWith
// ---------------------------------------------------------------------------

#[test]
fn starts_with_true() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(StartsWith('hello', 'hel'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn starts_with_false() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(StartsWith('hello', 'xyz'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

#[test]
fn ends_with_true() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(EndsWith('hello', 'llo'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn ends_with_false() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(EndsWith('hello', 'xyz'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

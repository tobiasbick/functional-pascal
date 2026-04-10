use super::super::super::support;

// ---------------------------------------------------------------------------
// Contains
// ---------------------------------------------------------------------------

#[test]
fn contains_found() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Contains('hello world', 'world'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn contains_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Contains('hello', 'xyz'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

#[test]
fn contains_empty_sub() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Contains('hello', ''))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

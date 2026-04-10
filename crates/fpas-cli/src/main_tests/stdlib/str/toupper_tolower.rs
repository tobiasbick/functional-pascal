use super::super::super::support;

// ---------------------------------------------------------------------------
// ToUpper / ToLower
// ---------------------------------------------------------------------------

#[test]
fn to_upper() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(ToUpper('hello'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "HELLO\n");
}

#[test]
fn to_upper_empty() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(ToUpper(''))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

#[test]
fn to_upper_already_upper() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(ToUpper('ABC'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "ABC\n");
}

#[test]
fn to_lower() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(ToLower('HELLO'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "hello\n");
}

#[test]
fn to_lower_empty() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(ToLower(''))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

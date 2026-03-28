use super::super::support;

// ---------------------------------------------------------------------------
// Write / WriteLn basics
// ---------------------------------------------------------------------------

#[test]
fn writeln_string() {
    let source = r#"program T;
uses Std.Console;
begin
  WriteLn('hello')
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "hello\n");
}

#[test]
fn writeln_integer() {
    let source = r#"program T;
uses Std.Console;
begin
  WriteLn(42)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "42\n");
}

#[test]
fn writeln_real() {
    let source = r#"program T;
uses Std.Console;
begin
  WriteLn(3.14)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert!(stdout.starts_with("3.14"), "got: {stdout}");
}

#[test]
fn writeln_boolean() {
    let source = r#"program T;
uses Std.Console;
begin
  WriteLn(true);
  WriteLn(false)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\nfalse\n");
}

#[test]
fn writeln_no_args_emits_newline() {
    let source = r#"program T;
uses Std.Console;
begin
  WriteLn
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

#[test]
fn writeln_multiple_args_variadic() {
    let source = r#"program T;
uses Std.Console;
begin
  WriteLn('val=', 42, ' ok=', true)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "val=42 ok=true\n");
}

#[test]
fn write_without_newline() {
    let source = r#"program T;
uses Std.Console;
begin
  Write('a');
  Write('b');
  WriteLn
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "ab\n");
}

// ---------------------------------------------------------------------------
// Fully qualified names
// ---------------------------------------------------------------------------

#[test]
fn fully_qualified_writeln() {
    let source = r#"program T;
uses Std.Console;
begin
  Std.Console.WriteLn('qualified')
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "qualified\n");
}

#[test]
fn fully_qualified_write() {
    let source = r#"program T;
uses Std.Console;
begin
  Std.Console.Write('a');
  Std.Console.WriteLn
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "a\n");
}

// ---------------------------------------------------------------------------
// Negative: missing uses
// ---------------------------------------------------------------------------

#[test]
fn writeln_without_uses_is_error() {
    let source = r#"program T;
begin
  WriteLn('hello')
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

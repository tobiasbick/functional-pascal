use super::super::support;

// ---------------------------------------------------------------------------
// Unwrap
// ---------------------------------------------------------------------------

#[test]
fn unwrap_ok() {
    let source = r#"program T;
uses Std.Console, Std.Result;
begin
  var R: Result of integer, string := Ok(42);
  WriteLn(Unwrap(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "42\n");
}

#[test]
fn unwrap_error_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Result;
begin
  var R: Result of integer, string := Error('oops');
  WriteLn(Unwrap(R))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

// ---------------------------------------------------------------------------
// UnwrapOr
// ---------------------------------------------------------------------------

#[test]
fn unwrap_or_ok_returns_value() {
    let source = r#"program T;
uses Std.Console, Std.Result;
begin
  var R: Result of integer, string := Ok(42);
  WriteLn(UnwrapOr(R, 0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "42\n");
}

#[test]
fn unwrap_or_error_returns_default() {
    let source = r#"program T;
uses Std.Console, Std.Result;
begin
  var R: Result of integer, string := Error('fail');
  WriteLn(UnwrapOr(R, 0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

// ---------------------------------------------------------------------------
// IsOk / IsError
// ---------------------------------------------------------------------------

#[test]
fn is_ok_with_ok() {
    let source = r#"program T;
uses Std.Console, Std.Result;
begin
  var R: Result of integer, string := Ok(42);
  WriteLn(IsOk(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn is_ok_with_error() {
    let source = r#"program T;
uses Std.Console, Std.Result;
begin
  var R: Result of integer, string := Error('fail');
  WriteLn(IsOk(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

#[test]
fn is_error_with_error() {
    let source = r#"program T;
uses Std.Console, Std.Result;
begin
  var R: Result of integer, string := Error('fail');
  WriteLn(IsError(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn is_error_with_ok() {
    let source = r#"program T;
uses Std.Console, Std.Result;
begin
  var R: Result of integer, string := Ok(42);
  WriteLn(IsError(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

// ---------------------------------------------------------------------------
// Fully qualified names
// ---------------------------------------------------------------------------

#[test]
fn fully_qualified_unwrap() {
    let source = r#"program T;
uses Std.Console, Std.Result;
begin
  var R: Result of integer, string := Ok(99);
  WriteLn(Std.Result.Unwrap(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "99\n");
}

#[test]
fn fully_qualified_is_ok() {
    let source = r#"program T;
uses Std.Console, Std.Result;
begin
  var R: Result of integer, string := Ok(1);
  WriteLn(Std.Result.IsOk(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

// ---------------------------------------------------------------------------
// Result with string values
// ---------------------------------------------------------------------------

#[test]
fn result_ok_string() {
    let source = r#"program T;
uses Std.Console, Std.Result;
begin
  var R: Result of string, string := Ok('hello');
  WriteLn(Unwrap(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "hello\n");
}

// ---------------------------------------------------------------------------
// Ambiguity with Std.Option
// ---------------------------------------------------------------------------

#[test]
fn result_and_option_require_qualified_unwrap() {
    let source = r#"program T;
uses Std.Console, Std.Result, Std.Option;
begin
  var R: Result of integer, string := Ok(42);
  var O: Option of integer := Some(7);
  WriteLn(Std.Result.Unwrap(R));
  WriteLn(Std.Option.Unwrap(O))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "42\n7\n");
}

#[test]
fn result_and_option_unqualified_unwrap_is_ambiguous() {
    let source = r#"program T;
uses Std.Console, Std.Result, Std.Option;
begin
  var R: Result of integer, string := Ok(42);
  WriteLn(Unwrap(R))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0, "should fail due to ambiguous Unwrap");
}

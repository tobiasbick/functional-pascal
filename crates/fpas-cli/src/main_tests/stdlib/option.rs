use super::super::support;

// ---------------------------------------------------------------------------
// Unwrap
// ---------------------------------------------------------------------------

#[test]
fn unwrap_some() {
    let source = r#"program T;
uses Std.Console, Std.Option;
begin
  var O: Option of integer := Some(7);
  WriteLn(Unwrap(O))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "7\n");
}

#[test]
fn unwrap_none_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Option;
begin
  var O: Option of integer := None;
  WriteLn(Unwrap(O))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

// ---------------------------------------------------------------------------
// UnwrapOr
// ---------------------------------------------------------------------------

#[test]
fn unwrap_or_some_returns_value() {
    let source = r#"program T;
uses Std.Console, Std.Option;
begin
  var O: Option of integer := Some(7);
  WriteLn(UnwrapOr(O, -1))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "7\n");
}

#[test]
fn unwrap_or_none_returns_default() {
    let source = r#"program T;
uses Std.Console, Std.Option;
begin
  var O: Option of integer := None;
  WriteLn(UnwrapOr(O, -1))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

// ---------------------------------------------------------------------------
// IsSome / IsNone
// ---------------------------------------------------------------------------

#[test]
fn is_some_with_some() {
    let source = r#"program T;
uses Std.Console, Std.Option;
begin
  var O: Option of integer := Some(7);
  WriteLn(IsSome(O))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn is_some_with_none() {
    let source = r#"program T;
uses Std.Console, Std.Option;
begin
  var O: Option of integer := None;
  WriteLn(IsSome(O))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

#[test]
fn is_none_with_none() {
    let source = r#"program T;
uses Std.Console, Std.Option;
begin
  var O: Option of integer := None;
  WriteLn(IsNone(O))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn is_none_with_some() {
    let source = r#"program T;
uses Std.Console, Std.Option;
begin
  var O: Option of integer := Some(7);
  WriteLn(IsNone(O))
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
uses Std.Console, Std.Option;
begin
  var O: Option of integer := Some(99);
  WriteLn(Std.Option.Unwrap(O))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "99\n");
}

#[test]
fn fully_qualified_unwrap_or() {
    let source = r#"program T;
uses Std.Console, Std.Option;
begin
  var O: Option of integer := None;
  WriteLn(Std.Option.UnwrapOr(O, 0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

// ---------------------------------------------------------------------------
// Option with string values
// ---------------------------------------------------------------------------

#[test]
fn option_some_string() {
    let source = r#"program T;
uses Std.Console, Std.Option;
begin
  var O: Option of string := Some('hello');
  WriteLn(Unwrap(O))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "hello\n");
}

#[test]
fn option_none_string_unwrap_or() {
    let source = r#"program T;
uses Std.Console, Std.Option;
begin
  var O: Option of string := None;
  WriteLn(UnwrapOr(O, 'default'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "default\n");
}

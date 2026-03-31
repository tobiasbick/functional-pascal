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
// Map
// ---------------------------------------------------------------------------

#[test]
fn map_ok_transforms_value() {
    let source = r#"program T;
uses Std.Console, Std.Result, Std.Conv;
function DoubleToStr(V: integer): string;
begin
  return IntToStr(V * 2)
end;
begin
  var R: Result of integer, string := Ok(21);
  var M: Result of string, string := Map(R, DoubleToStr);
  WriteLn(Unwrap(M))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "42\n");
}

#[test]
fn map_error_passes_through() {
    let source = r#"program T;
uses Std.Console, Std.Result;
function AlwaysOk(V: integer): string;
begin
  return 'ok'
end;
begin
  var R: Result of integer, string := Error('fail');
  var M: Result of string, string := Map(R, AlwaysOk);
  WriteLn(IsError(M))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn map_qualified_call() {
    let source = r#"program T;
uses Std.Console, Std.Result, Std.Conv;
function ToStr(V: integer): string;
begin
  return IntToStr(V)
end;
begin
  var R: Result of integer, string := Ok(5);
  var M: Result of string, string := Std.Result.Map(R, ToStr);
  WriteLn(Unwrap(M))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "5\n");
}

// ---------------------------------------------------------------------------
// AndThen
// ---------------------------------------------------------------------------

#[test]
fn and_then_ok_chains() {
    let source = r#"program T;
uses Std.Console, Std.Result, Std.Conv;
function PositiveToStr(V: integer): Result of string, string;
begin
  if V > 0 then return Ok(IntToStr(V))
  else return Error('non-positive')
end;
begin
  var R: Result of integer, string := Ok(10);
  var M: Result of string, string := AndThen(R, PositiveToStr);
  WriteLn(Unwrap(M))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "10\n");
}

#[test]
fn and_then_ok_produces_error() {
    let source = r#"program T;
uses Std.Console, Std.Result;
function PositiveOk(V: integer): Result of string, string;
begin
  if V > 0 then return Ok('ok')
  else return Error('non-positive')
end;
begin
  var R: Result of integer, string := Ok(-1);
  var M: Result of string, string := AndThen(R, PositiveOk);
  WriteLn(IsError(M))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn and_then_error_passes_through() {
    let source = r#"program T;
uses Std.Console, Std.Result;
function AlwaysOk(V: integer): Result of string, string;
begin
  return Ok('ok')
end;
begin
  var R: Result of integer, string := Error('original');
  var M: Result of string, string := AndThen(R, AlwaysOk);
  WriteLn(IsError(M))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

// ---------------------------------------------------------------------------
// OrElse
// ---------------------------------------------------------------------------

#[test]
fn or_else_error_recovers() {
    let source = r#"program T;
uses Std.Console, Std.Result;
function RecoverZero(E: string): Result of integer, string;
begin
  return Ok(0)
end;
begin
  var R: Result of integer, string := Error('oops');
  var M: Result of integer, string := OrElse(R, RecoverZero);
  WriteLn(Unwrap(M))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn or_else_ok_passes_through() {
    let source = r#"program T;
uses Std.Console, Std.Result;
function RecoverZero(E: string): Result of integer, string;
begin
  return Ok(0)
end;
begin
  var R: Result of integer, string := Ok(42);
  var M: Result of integer, string := OrElse(R, RecoverZero);
  WriteLn(Unwrap(M))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "42\n");
}

#[test]
fn or_else_error_to_error() {
    let source = r#"program T;
uses Std.Console, Std.Result;
function ReplaceError(E: string): Result of integer, string;
begin
  return Error('second')
end;
begin
  var R: Result of integer, string := Error('first');
  var M: Result of integer, string := OrElse(R, ReplaceError);
  WriteLn(IsError(M))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

// ---------------------------------------------------------------------------
// Combinator chains
// ---------------------------------------------------------------------------

#[test]
fn map_then_and_then_chain() {
    let source = r#"program T;
uses Std.Console, Std.Result, Std.Conv;
function Double(V: integer): integer;
begin
  return V * 2
end;
function PositiveToStr(V: integer): Result of string, string;
begin
  if V > 0 then return Ok(IntToStr(V))
  else return Error('non-positive')
end;
begin
  var R: Result of integer, string := Ok(5);
  var Doubled: Result of integer, string := Map(R, Double);
  var Final: Result of string, string := AndThen(Doubled, PositiveToStr);
  WriteLn(Unwrap(Final))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "10\n");
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

use super::{compile_and_run, compile_run_err};
#[test]
fn result_ok_and_writeln() {
    let out = compile_and_run(
        "program T;
var R: Result of integer, string := Ok(42);
begin
  Std.Console.WriteLn(Std.Result.Unwrap(R))
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}
#[test]
fn result_err_and_is_err() {
    let out = compile_and_run(
        "program T;
var R: Result of integer, string := Error('oops');
begin
  if Std.Result.IsError(R) then
    Std.Console.WriteLn('error')
  else
    Std.Console.WriteLn('ok')
end.",
    );
    assert_eq!(out.lines, vec!["error"]);
}
#[test]
fn result_is_ok() {
    let out = compile_and_run(
        "program T;
var R: Result of integer, string := Ok(10);
begin
  if Std.Result.IsOk(R) then
    Std.Console.WriteLn('yes')
  else
    Std.Console.WriteLn('no')
end.",
    );
    assert_eq!(out.lines, vec!["yes"]);
}
#[test]
fn result_unwrap_or() {
    let out = compile_and_run(
        "program T;
var R: Result of integer, string := Error('fail');
begin
  Std.Console.WriteLn(Std.Result.UnwrapOr(R, 99))
end.",
    );
    assert_eq!(out.lines, vec!["99"]);
}
#[test]
fn result_unwrap_err_panics() {
    let msg = compile_run_err(
        "program T;
var R: Result of integer, string := Error('bad');
begin
  Std.Console.WriteLn(Std.Result.Unwrap(R))
end.",
    );
    assert!(msg.contains("Unwrap"), "Expected unwrap error, got: {msg}");
}

#[test]
fn result_unwrap_or_ok_returns_value() {
    let out = compile_and_run(
        "program T;
var R: Result of integer, string := Ok(7);
begin
  Std.Console.WriteLn(Std.Result.UnwrapOr(R, 99))
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

#[test]
fn result_is_ok_on_error() {
    let out = compile_and_run(
        "program T;
var R: Result of integer, string := Error('e');
begin
  if Std.Result.IsOk(R) then
    Std.Console.WriteLn('yes')
  else
    Std.Console.WriteLn('no')
end.",
    );
    assert_eq!(out.lines, vec!["no"]);
}

#[test]
fn result_is_error_on_ok() {
    let out = compile_and_run(
        "program T;
var R: Result of integer, string := Ok(1);
begin
  if Std.Result.IsError(R) then
    Std.Console.WriteLn('yes')
  else
    Std.Console.WriteLn('no')
end.",
    );
    assert_eq!(out.lines, vec!["no"]);
}

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

// ── Edge cases ──────────────────────────────────────────────────────────

#[test]
fn empty_error_message_string() {
    let out = compile_and_run(
        "program T;
var R: Result of integer, string := Error('');
begin
  if Std.Result.IsError(R) then
    Std.Console.WriteLn('is error')
  else
    Std.Console.WriteLn('not error')
end.",
    );
    assert_eq!(out.lines, vec!["is error"]);
}

#[test]
fn result_of_boolean_ok() {
    let out = compile_and_run(
        "program T;
var R: Result of boolean, string := Ok(true);
begin
  Std.Console.WriteLn(Std.Result.Unwrap(R))
end.",
    );
    assert_eq!(out.lines, vec!["true"]);
}

#[test]
fn result_of_boolean_error() {
    let out = compile_and_run(
        "program T;
var R: Result of boolean, string := Error('nope');
begin
  Std.Console.WriteLn(Std.Result.UnwrapOr(R, false))
end.",
    );
    assert_eq!(out.lines, vec!["false"]);
}

#[test]
fn result_in_for_loop() {
    let out = compile_and_run(
        "program T;
function SafeDiv(A: integer; B: integer): Result of integer, string;
begin
  if B = 0 then return Error('zero')
  else return Ok(A div B)
end;
begin
  for I: integer := 1 to 3 do
  begin
    case SafeDiv(10, I) of
      Ok(V):    Std.Console.WriteLn(V);
      Error(E): Std.Console.WriteLn(E)
    end
  end
end.",
    );
    assert_eq!(out.lines, vec!["10", "5", "3"]);
}

#[test]
fn result_in_while_loop() {
    let out = compile_and_run(
        "program T;
function Check(X: integer): Result of integer, string;
begin
  if X > 3 then return Error('too big')
  else return Ok(X * 10)
end;
mutable var I: integer := 1;
begin
  while I <= 5 do
  begin
    case Check(I) of
      Ok(V):    Std.Console.WriteLn(V);
      Error(E): Std.Console.WriteLn(E)
    end;
    I := I + 1
  end
end.",
    );
    assert_eq!(out.lines, vec!["10", "20", "30", "too big", "too big"]);
}

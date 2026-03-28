use super::{compile_and_run, compile_run_err};
#[test]
fn option_some_and_unwrap() {
    let out = compile_and_run(
        "program T;
var O: Option of integer := Some(7);
begin
  Std.Console.WriteLn(Std.Option.Unwrap(O))
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}
#[test]
fn option_none_and_is_none() {
    let out = compile_and_run(
        "program T;
var O: Option of integer := None;
begin
  if Std.Option.IsNone(O) then
    Std.Console.WriteLn('none')
  else
    Std.Console.WriteLn('some')
end.",
    );
    assert_eq!(out.lines, vec!["none"]);
}
#[test]
fn option_is_some() {
    let out = compile_and_run(
        "program T;
var O: Option of integer := Some(1);
begin
  if Std.Option.IsSome(O) then
    Std.Console.WriteLn('yes')
  else
    Std.Console.WriteLn('no')
end.",
    );
    assert_eq!(out.lines, vec!["yes"]);
}
#[test]
fn option_unwrap_or() {
    let out = compile_and_run(
        "program T;
var O: Option of integer := None;
begin
  Std.Console.WriteLn(Std.Option.UnwrapOr(O, 42))
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}
#[test]
fn option_unwrap_none_panics() {
    let msg = compile_run_err(
        "program T;
var O: Option of integer := None;
begin
  Std.Console.WriteLn(Std.Option.Unwrap(O))
end.",
    );
    assert!(msg.contains("Unwrap"), "Expected unwrap error, got: {msg}");
}

#[test]
fn option_unwrap_or_some_returns_value() {
    let out = compile_and_run(
        "program T;
var O: Option of integer := Some(7);
begin
  Std.Console.WriteLn(Std.Option.UnwrapOr(O, 99))
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

#[test]
fn option_is_some_on_none() {
    let out = compile_and_run(
        "program T;
var O: Option of integer := None;
begin
  if Std.Option.IsSome(O) then
    Std.Console.WriteLn('yes')
  else
    Std.Console.WriteLn('no')
end.",
    );
    assert_eq!(out.lines, vec!["no"]);
}

#[test]
fn option_is_none_on_some() {
    let out = compile_and_run(
        "program T;
var O: Option of integer := Some(1);
begin
  if Std.Option.IsNone(O) then
    Std.Console.WriteLn('yes')
  else
    Std.Console.WriteLn('no')
end.",
    );
    assert_eq!(out.lines, vec!["no"]);
}

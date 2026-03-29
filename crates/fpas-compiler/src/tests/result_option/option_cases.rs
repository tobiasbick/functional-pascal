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

// ── Edge cases ──────────────────────────────────────────────────────────

#[test]
fn option_of_real() {
    let out = compile_and_run(
        "program T;
var O: Option of real := Some(3.14);
begin
  case O of
    Some(V): Std.Console.WriteLn(V);
    None:    Std.Console.WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["3.14"]);
}

#[test]
fn option_in_for_loop() {
    let out = compile_and_run(
        "program T;
function MaybeEven(X: integer): Option of integer;
begin
  if X mod 2 = 0 then return Some(X)
  else return None
end;
begin
  for I: integer := 1 to 4 do
  begin
    case MaybeEven(I) of
      Some(V): Std.Console.WriteLn(V);
      None:    Std.Console.WriteLn('odd')
    end
  end
end.",
    );
    assert_eq!(out.lines, vec!["odd", "2", "odd", "4"]);
}

#[test]
fn option_in_while_loop() {
    let out = compile_and_run(
        "program T;
function Half(X: integer): Option of integer;
begin
  if X mod 2 = 0 then return Some(X div 2)
  else return None
end;
mutable var N: integer := 16;
begin
  while Std.Option.IsSome(Half(N)) do
  begin
    N := Std.Option.Unwrap(Half(N));
    Std.Console.WriteLn(N)
  end
end.",
    );
    assert_eq!(out.lines, vec!["8", "4", "2", "1"]);
}

#[test]
fn option_none_as_function_return() {
    let out = compile_and_run(
        "program T;
function AlwaysNone(): Option of string;
begin
  return None
end;
begin
  case AlwaysNone() of
    Some(S): Std.Console.WriteLn(S);
    None:    Std.Console.WriteLn('nothing')
  end
end.",
    );
    assert_eq!(out.lines, vec!["nothing"]);
}

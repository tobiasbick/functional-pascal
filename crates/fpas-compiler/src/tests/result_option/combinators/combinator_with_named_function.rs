use super::super::{compile_and_run, compile_err};
// ── Combinator with named function ──────────────────────────────────────

#[test]
fn result_map_with_named_function() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;

function DoubleIt(V: integer): integer;
begin
  return V * 2
end;

begin
  var R: Result of integer, string := Ok(21);
  var M: Result of integer, string := Map(R, DoubleIt);
  case M of
    Ok(V): WriteLn(IntToStr(V));
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn option_and_then_with_named_function() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;

function SafeHalf(V: integer): Option of integer;
begin
  if V mod 2 = 0 then return Some(V div 2)
  else return None
end;

begin
  var O: Option of integer := Some(10);
  var M: Option of integer := AndThen(O, SafeHalf);
  case M of
    Some(V): WriteLn(IntToStr(V));
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn option_and_then_named_function_returns_none() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;

function SafeHalf(V: integer): Option of integer;
begin
  if V mod 2 = 0 then return Some(V div 2)
  else return None
end;

begin
  var O: Option of integer := Some(7);
  var M: Option of integer := AndThen(O, SafeHalf);
  case M of
    Some(V): WriteLn(IntToStr(V));
    None: WriteLn('odd')
  end
end.",
    );
    assert_eq!(out.lines, vec!["odd"]);
}

use super::super::{compile_and_run, compile_err};
// ── Option.AndThen ──────────────────────────────────────────────────────

#[test]
fn option_and_then_some_chains() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function PositiveToStr(V: integer): Option of string;
begin
  if V > 0 then return Some(IntToStr(V))
  else return None
end;
begin
  var O: Option of integer := Some(5);
  var M: Option of string := AndThen(O, PositiveToStr);
  case M of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn option_and_then_some_returns_none() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function PositiveToStr(V: integer): Option of string;
begin
  if V > 0 then return Some(IntToStr(V))
  else return None
end;
begin
  var O: Option of integer := Some(-1);
  var M: Option of string := AndThen(O, PositiveToStr);
  case M of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["none"]);
}

#[test]
fn option_and_then_none_passes_through() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function ToSomeStr(V: integer): Option of string;
begin
  return Some(IntToStr(V))
end;
begin
  var O: Option of integer := None;
  var M: Option of string := AndThen(O, ToSomeStr);
  case M of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["none"]);
}

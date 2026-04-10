use super::super::{compile_and_run, compile_err};
// ── Option.Map ──────────────────────────────────────────────────────────

#[test]
fn option_map_some_transforms_value() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function TripleToStr(V: integer): string;
begin
  return IntToStr(V * 3)
end;
begin
  var O: Option of integer := Some(7);
  var M: Option of string := Map(O, TripleToStr);
  case M of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["21"]);
}

#[test]
fn option_map_none_passes_through() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function ToStr(V: integer): string;
begin
  return IntToStr(V)
end;
begin
  var O: Option of integer := None;
  var M: Option of string := Map(O, ToStr);
  case M of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["none"]);
}

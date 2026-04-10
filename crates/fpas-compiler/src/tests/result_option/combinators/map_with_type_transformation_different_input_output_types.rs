use super::super::{compile_and_run, compile_err};
// ── Map with type transformation (different input/output types) ─────────

#[test]
fn result_map_integer_to_boolean() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result;
function IsPositive(V: integer): boolean;
begin
  return V > 0
end;
begin
  var R: Result of integer, string := Ok(0);
  var M: Result of boolean, string := Map(R, IsPositive);
  case M of
    Ok(B): if B then WriteLn('positive') else WriteLn('non-positive');
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["non-positive"]);
}

#[test]
fn option_map_string_to_integer() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function ParseAndIncr(S: string): integer;
begin
  return StrToInt(S) + 1
end;
begin
  var O: Option of string := Some('123');
  var M: Option of integer := Map(O, ParseAndIncr);
  case M of
    Some(V): WriteLn(IntToStr(V));
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["124"]);
}

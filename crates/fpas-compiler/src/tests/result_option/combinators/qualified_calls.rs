use super::super::compile_and_run;
// ── Qualified calls ─────────────────────────────────────────────────────

#[test]
fn result_map_qualified_call() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function MulSeven(V: integer): integer;
begin
  return V * 7
end;
begin
  var R: Result of integer, string := Ok(6);
  var M: Result of integer, string := Std.Result.Map(R, MulSeven);
  case M of
    Ok(V): WriteLn(IntToStr(V));
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn option_and_then_qualified_call() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function ToSomeStr(V: integer): Option of string;
begin
  return Some(IntToStr(V))
end;
begin
  var O: Option of integer := Some(10);
  var M: Option of string := Std.Option.AndThen(O, ToSomeStr);
  case M of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

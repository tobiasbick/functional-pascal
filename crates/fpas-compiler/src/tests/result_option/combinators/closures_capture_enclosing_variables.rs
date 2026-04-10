use super::super::compile_and_run;
// ── Closures capture enclosing variables ────────────────────────────────

#[test]
fn result_map_closure_captures_variable() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function MultiplyByHundred(V: integer): integer;
begin
  return V * 100
end;
begin
  var R: Result of integer, string := Ok(3);
  var M: Result of integer, string := Map(R, MultiplyByHundred);
  case M of
    Ok(V): WriteLn(IntToStr(V));
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["300"]);
}

#[test]
fn option_map_closure_captures_variable() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function PrefixedToStr(V: integer): string;
begin
  return 'val=' + IntToStr(V)
end;
begin
  var O: Option of integer := Some(42);
  var M: Option of string := Map(O, PrefixedToStr);
  case M of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["val=42"]);
}

use super::super::{compile_and_run, compile_err};
// ── Identity map (callback returns input unchanged) ─────────────────────

#[test]
fn result_map_identity() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function Identity(V: integer): integer;
begin
  return V
end;
begin
  var R: Result of integer, string := Ok(42);
  var M: Result of integer, string := Map(R, Identity);
  case M of
    Ok(V): WriteLn(IntToStr(V));
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn option_map_identity() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function Identity(V: integer): integer;
begin
  return V
end;
begin
  var O: Option of integer := Some(7);
  var M: Option of integer := Map(O, Identity);
  case M of
    Some(V): WriteLn(IntToStr(V));
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

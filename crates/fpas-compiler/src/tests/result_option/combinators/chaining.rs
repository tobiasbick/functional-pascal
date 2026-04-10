use super::super::compile_and_run;
// ── Chaining ────────────────────────────────────────────────────────────

#[test]
fn result_map_and_then_chain() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function MulTen(V: integer): integer;
begin
  return V * 10
end;
function WrapInOk(V: integer): Result of string, string;
begin
  return Ok(IntToStr(V))
end;
begin
  var R: Result of integer, string := Ok(5);
  var Step1: Result of integer, string := Map(R, MulTen);
  var Step2: Result of string, string := AndThen(Step1, WrapInOk);
  case Step2 of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["50"]);
}

#[test]
fn option_map_and_then_chain() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function AddSeven(V: integer): integer;
begin
  return V + 7
end;
function WrapInSome(V: integer): Option of string;
begin
  return Some(IntToStr(V))
end;
begin
  var O: Option of integer := Some(3);
  var Step1: Option of integer := Map(O, AddSeven);
  var Step2: Option of string := AndThen(Step1, WrapInSome);
  case Step2 of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

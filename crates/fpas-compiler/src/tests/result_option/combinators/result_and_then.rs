use super::super::{compile_and_run, compile_err};
// ── Result.AndThen ──────────────────────────────────────────────────────

#[test]
fn result_and_then_ok_chains() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function PositiveToStr(V: integer): Result of string, string;
begin
  if V > 0 then return Ok(IntToStr(V))
  else return Error('non-positive')
end;
begin
  var R: Result of integer, string := Ok(10);
  var M: Result of string, string := AndThen(R, PositiveToStr);
  case M of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

#[test]
fn result_and_then_ok_produces_error() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function PositiveToStr(V: integer): Result of string, string;
begin
  if V > 0 then return Ok(IntToStr(V))
  else return Error('non-positive')
end;
begin
  var R: Result of integer, string := Ok(-5);
  var M: Result of string, string := AndThen(R, PositiveToStr);
  case M of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["non-positive"]);
}

#[test]
fn result_and_then_error_passes_through() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function WrapOk(V: integer): Result of string, string;
begin
  return Ok(IntToStr(V))
end;
begin
  var R: Result of integer, string := Error('early');
  var M: Result of string, string := AndThen(R, WrapOk);
  case M of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["early"]);
}

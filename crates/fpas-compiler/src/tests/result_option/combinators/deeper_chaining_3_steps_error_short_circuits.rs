use super::super::{compile_and_run, compile_err};
// ── Deeper chaining (3+ steps, error short-circuits) ────────────────────

#[test]
fn result_three_step_chain_ok() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function AddThree(V: integer): integer;
begin
  return V + 3
end;
function MulTen(V: integer): integer;
begin
  return V * 10
end;
function WrapResult(V: integer): Result of string, string;
begin
  return Ok('result=' + IntToStr(V))
end;
begin
  var R: Result of integer, string := Ok(2);
  var S1: Result of integer, string := Map(R, AddThree);
  var S2: Result of integer, string := Map(S1, MulTen);
  var S3: Result of string, string := AndThen(S2, WrapResult);
  case S3 of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["result=50"]);
}

#[test]
fn result_three_step_chain_error_short_circuits() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function AddThree(V: integer): integer;
begin
  return V + 3
end;
function MulTen(V: integer): integer;
begin
  return V * 10
end;
function WrapResult(V: integer): Result of string, string;
begin
  return Ok(IntToStr(V))
end;
begin
  var R: Result of integer, string := Error('boom');
  var S1: Result of integer, string := Map(R, AddThree);
  var S2: Result of integer, string := Map(S1, MulTen);
  var S3: Result of string, string := AndThen(S2, WrapResult);
  case S3 of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["boom"]);
}

#[test]
fn option_three_step_chain_none_short_circuits() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function AddOne(V: integer): integer;
begin
  return V + 1
end;
function MulTwo(V: integer): integer;
begin
  return V * 2
end;
function WrapSome(V: integer): Option of string;
begin
  return Some(IntToStr(V))
end;
begin
  var O: Option of integer := None;
  var S1: Option of integer := Map(O, AddOne);
  var S2: Option of integer := Map(S1, MulTwo);
  var S3: Option of string := AndThen(S2, WrapSome);
  case S3 of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["none"]);
}

use super::super::compile_and_run;
// ── Combined Result + Option combinators ────────────────────────────────

#[test]
fn result_and_option_combinators_interleaved() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Option, Std.Conv;

function ParsePositive(S: string): Result of integer, string;
begin
  var N: integer := StrToInt(S);
  if N > 0 then return Ok(N)
  else return Error('non-positive')
end;

function ToSomeIfBig(V: integer): Option of integer;
begin
  if V > 10 then return Some(V)
  else return None
end;

begin
  var R: Result of integer, string := ParsePositive('42');
  var Mapped: Result of Option of integer, string := Std.Result.Map(R, ToSomeIfBig);
  case Mapped of
    Ok(Inner):
      case Inner of
        Some(V): WriteLn('big: ' + IntToStr(V));
        None: WriteLn('small')
      end;
    Error(E): WriteLn('error: ' + E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["big: 42"]);
}

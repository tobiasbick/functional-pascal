use super::super::{compile_and_run, compile_err};
// ── Result.OrElse ───────────────────────────────────────────────────────

#[test]
fn result_or_else_ok_passes_through() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function RecoverZero(E: string): Result of integer, string;
begin
  return Ok(0)
end;
begin
  var R: Result of integer, string := Ok(42);
  var M: Result of integer, string := OrElse(R, RecoverZero);
  case M of
    Ok(V): WriteLn(IntToStr(V));
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn result_or_else_error_recovers() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function RecoverNinetyNine(E: string): Result of integer, string;
begin
  return Ok(99)
end;
begin
  var R: Result of integer, string := Error('oops');
  var M: Result of integer, string := OrElse(R, RecoverNinetyNine);
  case M of
    Ok(V): WriteLn(IntToStr(V));
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["99"]);
}

use super::super::{compile_and_run, compile_err};
// ── Error content preservation through try ──────────────────────────────

#[test]
fn try_preserves_error_content() {
    let out = compile_and_run(
        "program T;
function Inner(): Result of integer, string;
begin
  return Error('specific error')
end;
function Outer(): Result of string, string;
begin
  var V: integer := try Inner();
  return Ok(Std.Conv.IntToStr(V))
end;
begin
  case Outer() of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["specific error"]);
}

use super::super::{compile_and_run, compile_err};
// ── Try in expression context ───────────────────────────────────────────

#[test]
fn try_result_used_in_arithmetic() {
    let out = compile_and_run(
        "program T;
function GetVal(): Result of integer, string;
begin
  return Ok(5)
end;
function Compute(): Result of integer, string;
begin
  return Ok(try GetVal() * 3)
end;
begin
  Std.Console.WriteLn(Std.Result.Unwrap(Compute()))
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}

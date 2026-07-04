/// Tests for basic function and procedure declarations.
///
/// **Documentation:** [docs/pascal/language/functions/README.md](docs/pascal/language/functions/README.md)
use super::super::*;

// ═══════════════════════════════════════════════════════════════
// EARLY RETURN
// ═══════════════════════════════════════════════════════════════

#[test]
fn early_return_from_loop() {
    let out = compile_and_run(
        "\
program EarlyReturn;
uses Std.Console, Std.Array;

function IndexOf(Items: array of string; Target: string): integer;
begin
  for I: integer := 0 to Std.Array.Length(Items) - 1 do
  begin
    if Items[I] = Target then
      return I
  end;
  return -1
end;

begin
  WriteLn(IndexOf(['a', 'b', 'c'], 'b'));
  WriteLn(IndexOf(['a', 'b', 'c'], 'z'))
end.",
    );
    assert_eq!(out.lines, vec!["1", "-1"]);
}

#[test]
fn early_return_skips_remaining_code() {
    let out = compile_and_run(
        "\
program EarlySkip;
uses Std.Console;

function Check(N: integer): string;
begin
  if N < 0 then
    return 'negative';
  if N = 0 then
    return 'zero';
  return 'positive'
end;

begin
  WriteLn(Check(-5));
  WriteLn(Check(0));
  WriteLn(Check(7))
end.",
    );
    assert_eq!(out.lines, vec!["negative", "zero", "positive"]);
}

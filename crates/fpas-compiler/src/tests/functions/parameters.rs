/// Tests for basic function and procedure declarations.
///
/// **Documentation:** [docs/pascal/language/functions/README.md](docs/pascal/language/functions/README.md)
use super::super::*;

// ═══════════════════════════════════════════════════════════════
// PARAMETERS
// ═══════════════════════════════════════════════════════════════

#[test]
fn multi_param_function_clamp() {
    let out = compile_and_run(
        "\
program ClampTest;
uses Std.Console;

function Clamp(Value: integer; Min: integer; Max: integer): integer;
begin
  if Value < Min then
    return Min
  else if Value > Max then
    return Max
  else
    return Value
end;

begin
  WriteLn(Clamp(150, 0, 100));
  WriteLn(Clamp(-5, 0, 100));
  WriteLn(Clamp(50, 0, 100))
end.",
    );
    assert_eq!(out.lines, vec!["100", "0", "50"]);
}

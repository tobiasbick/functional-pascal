use super::*;

#[test]
fn while_continue_skips_iteration() {
    let out = compile_and_run(
        "\
program WhileContinue;
begin
  mutable var I: integer := 0;
  while I < 5 do
  begin
    I := I + 1;
    if I mod 2 = 0 then
      continue;
    Std.Console.WriteLn(I)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "3", "5"]);
}

#[test]
fn while_break_and_continue() {
    let out = compile_and_run(
        "\
program WhileBreakContinue;
begin
  mutable var I: integer := 0;
  while true do
  begin
    I := I + 1;
    if I mod 2 = 0 then
      continue;
    if I > 7 then
      break;
    Std.Console.WriteLn(I)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "3", "5", "7"]);
}

#[test]
fn while_true_conditional_break() {
    let out = compile_and_run(
        "\
program WhileTrueBreak;
begin
  mutable var I: integer := 10;
  while true do
  begin
    I := I - 3;
    if I <= 0 then
      break
  end;
  Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["-2"]);
}

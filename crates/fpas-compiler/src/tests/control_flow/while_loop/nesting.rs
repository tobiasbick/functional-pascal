use super::*;

#[test]
fn while_break_exits_inner_only() {
    let out = compile_and_run(
        "\
program WhileBreakInner;
begin
  mutable var I: integer := 0;
  while I < 3 do
  begin
    mutable var J: integer := 0;
    while true do
    begin
      if J = 2 then
        break;
      J := J + 1
    end;
    Std.Console.WriteLn(J);
    I := I + 1
  end
end.",
    );
    assert_eq!(out.lines, vec!["2", "2", "2"]);
}

#[test]
fn while_continue_inner_only() {
    let out = compile_and_run(
        "\
program WhileContinueInner;
begin
  mutable var I: integer := 0;
  while I < 2 do
  begin
    mutable var J: integer := 0;
    mutable var Sum: integer := 0;
    while J < 5 do
    begin
      J := J + 1;
      if J mod 2 = 0 then
        continue;
      Sum := Sum + J
    end;
    Std.Console.WriteLn(Sum);
    I := I + 1
  end
end.",
    );
    assert_eq!(out.lines, vec!["9", "9"]);
}

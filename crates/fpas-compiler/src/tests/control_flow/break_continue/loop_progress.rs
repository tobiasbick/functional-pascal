use super::*;

#[test]
fn for_break_at_last_value() {
    let out = compile_and_run(
        "\
program ForBrkLast;
begin
  mutable var Last: integer := 0;
  for I: integer := 1 to 5 do
  begin
    Last := I;
    if I = 5 then
      break
  end;
  Std.Console.WriteLn(Last)
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn for_downto_break_at_last_value() {
    let out = compile_and_run(
        "\
program ForDownBrkLast;
begin
  mutable var Last: integer := 0;
  for I: integer := 5 downto 1 do
  begin
    Last := I;
    if I = 1 then
      break
  end;
  Std.Console.WriteLn(Last)
end.",
    );
    assert_eq!(out.lines, vec!["1"]);
}

#[test]
fn continue_at_end_of_body_is_noop() {
    let out = compile_and_run(
        "\
program ContEnd;
begin
  for I: integer := 1 to 3 do
  begin
    Std.Console.WriteLn(I);
    continue
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "3"]);
}

#[test]
fn continue_at_end_of_while_body() {
    let out = compile_and_run(
        "\
program ContEndWhile;
begin
  mutable var I: integer := 0;
  while I < 3 do
  begin
    I := I + 1;
    Std.Console.WriteLn(I);
    continue
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "3"]);
}

#[test]
fn while_break_preserves_state_after_loop() {
    let out = compile_and_run(
        "\
program WhileBrkState;
begin
  mutable var I: integer := 0;
  mutable var Total: integer := 0;
  while true do
  begin
    I := I + 1;
    Total := Total + I;
    if I = 4 then
      break
  end;
  Std.Console.WriteLn(I);
  Std.Console.WriteLn(Total)
end.",
    );
    assert_eq!(out.lines, vec!["4", "10"]);
}

#[test]
fn for_continue_still_increments() {
    let out = compile_and_run(
        "\
program ForContIncr;
begin
  mutable var Sum: integer := 0;
  for I: integer := 1 to 5 do
  begin
    continue;
    Sum := Sum + I
  end;
  Std.Console.WriteLn(Sum)
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn for_downto_continue_still_decrements() {
    let out = compile_and_run(
        "\
program ForDownContDecr;
begin
  mutable var Count: integer := 0;
  for I: integer := 5 downto 1 do
  begin
    Count := Count + 1;
    continue
  end;
  Std.Console.WriteLn(Count)
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn break_after_output() {
    let out = compile_and_run(
        "\
program BrkAfterOut;
begin
  for I: integer := 1 to 10 do
  begin
    Std.Console.WriteLn(I);
    if I = 3 then
      break
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "3"]);
}

#[test]
fn many_continues_in_loop() {
    let out = compile_and_run(
        "\
program ManyCont;
begin
  for I: integer := 1 to 10 do
  begin
    if I = 1 then continue;
    if I = 3 then continue;
    if I = 5 then continue;
    if I = 7 then continue;
    if I = 9 then continue;
    Std.Console.WriteLn(I)
  end
end.",
    );
    assert_eq!(out.lines, vec!["2", "4", "6", "8", "10"]);
}

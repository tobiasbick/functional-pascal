use super::*;

#[test]
fn for_to_break() {
    let out = compile_and_run(
        "\
program ForToBreak;
begin
  for I: integer := 1 to 10 do
  begin
    if I = 4 then break;
    Std.Console.WriteLn(I)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "3"]);
}

#[test]
fn for_downto_break() {
    let out = compile_and_run(
        "\
program ForDowntoBreak;
begin
  for I: integer := 10 downto 1 do
  begin
    if I = 7 then break;
    Std.Console.WriteLn(I)
  end
end.",
    );
    assert_eq!(out.lines, vec!["10", "9", "8"]);
}

#[test]
fn for_to_break_first_iteration() {
    let out = compile_and_run(
        "\
program ForBreakFirst;
begin
  mutable var Count: integer := 0;
  for I: integer := 1 to 100 do
  begin
    break;
    Count := Count + 1
  end;
  Std.Console.WriteLn(Count)
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn for_to_continue() {
    let out = compile_and_run(
        "\
program ForToContinue;
begin
  for I: integer := 1 to 6 do
  begin
    if I mod 2 = 0 then continue;
    Std.Console.WriteLn(I)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "3", "5"]);
}

#[test]
fn for_downto_continue() {
    let out = compile_and_run(
        "\
program ForDowntoContinue;
begin
  for I: integer := 6 downto 1 do
  begin
    if I mod 2 = 0 then continue;
    Std.Console.WriteLn(I)
  end
end.",
    );
    assert_eq!(out.lines, vec!["5", "3", "1"]);
}

#[test]
fn for_to_continue_all_skipped() {
    let out = compile_and_run(
        "\
program ForContinueAll;
begin
  mutable var Reached: integer := 0;
  for I: integer := 1 to 5 do
  begin
    continue;
    Reached := Reached + 1
  end;
  Std.Console.WriteLn(Reached)
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn for_to_break_and_continue() {
    let out = compile_and_run(
        "\
program ForBreakContinue;
begin
  for I: integer := 1 to 100 do
  begin
    if I mod 2 = 0 then
      continue;
    if I > 50 then
      break;
    Std.Console.WriteLn(I)
  end
end.",
    );
    let expected: Vec<String> = (1..=49).step_by(2).map(|n| n.to_string()).collect();
    assert_eq!(out.lines, expected);
}

#[test]
fn code_after_for_to_executes() {
    let out = compile_and_run(
        "\
program ForAfter;
begin
  for I: integer := 1 to 3 do
    Std.Console.WriteLn(I);
  Std.Console.WriteLn('done')
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "3", "done"]);
}

#[test]
fn code_after_zero_iteration_for_executes() {
    let out = compile_and_run(
        "\
program ForZeroAfter;
begin
  for I: integer := 10 to 1 do
    Std.Console.WriteLn(I);
  Std.Console.WriteLn('still runs')
end.",
    );
    assert_eq!(out.lines, vec!["still runs"]);
}

#[test]
fn code_after_broken_for_executes() {
    let out = compile_and_run(
        "\
program ForBrokenAfter;
begin
  for I: integer := 1 to 100 do
    break;
  Std.Console.WriteLn('after break')
end.",
    );
    assert_eq!(out.lines, vec!["after break"]);
}

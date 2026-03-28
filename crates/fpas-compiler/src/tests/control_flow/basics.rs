use super::*;
#[test]
fn if_then() {
    let out = compile_and_run(
        "\
program IfTest;
begin
  if true then
    Std.Console.WriteLn('yes')
end.",
    );
    assert_eq!(out.lines, vec!["yes"]);
}
#[test]
fn if_then_else() {
    let out = compile_and_run(
        "\
program IfElse;
begin
  if false then
    Std.Console.WriteLn('no')
  else
    Std.Console.WriteLn('yes')
end.",
    );
    assert_eq!(out.lines, vec!["yes"]);
}
#[test]
fn while_loop() {
    let out = compile_and_run(
        "\
program WhileTest;
begin
  mutable var I: integer := 0;
  while I < 3 do
  begin
    Std.Console.WriteLn(I);
    I := I + 1
  end
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "2"]);
}
#[test]
fn for_loop_to() {
    let out = compile_and_run(
        "\
program ForTest;
begin
  for I: integer := 1 to 3 do
    Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "3"]);
}
#[test]
fn for_loop_downto() {
    let out = compile_and_run(
        "\
program ForDown;
begin
  for I: integer := 3 downto 1 do
    Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["3", "2", "1"]);
}
#[test]
fn repeat_until() {
    let out = compile_and_run(
        "\
program RepeatTest;
begin
  mutable var I: integer := 0;
  repeat
    Std.Console.WriteLn(I);
    I := I + 1
  until I = 3
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "2"]);
}

use super::*;
// nested loops: same type.
#[test]
fn nested_for_in_for() {
    let out = compile_and_run(
        "\
program ForFor;
begin
  for I: integer := 1 to 2 do
    for J: integer := 1 to 3 do
      Std.Console.WriteLn(I * 10 + J)
end.",
    );
    assert_eq!(out.lines, vec!["11", "12", "13", "21", "22", "23"]);
}
#[test]
fn nested_while_in_while() {
    let out = compile_and_run(
        "\
program WhileWhile;
begin
  mutable var I: integer := 0;
  while I < 2 do
  begin
    mutable var J: integer := 0;
    while J < 3 do
    begin
      Std.Console.WriteLn(I * 10 + J);
      J := J + 1
    end;
    I := I + 1
  end
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "2", "10", "11", "12"]);
}
#[test]
fn nested_repeat_in_repeat() {
    let out = compile_and_run(
        "\
program RepeatRepeat;
begin
  mutable var I: integer := 0;
  repeat
    mutable var J: integer := 0;
    repeat
      Std.Console.WriteLn(I * 10 + J);
      J := J + 1
    until J = 2;
    I := I + 1
  until I = 3
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "10", "11", "20", "21"]);
}

use super::*;
// nested loops: mixed types.
#[test]
fn for_in_repeat() {
    let out = compile_and_run(
        "\
program ForInRepeat;
begin
  mutable var N: integer := 0;
  repeat
    for J: integer := 1 to 3 do
      Std.Console.WriteLn(N * 10 + J);
    N := N + 1
  until N = 2
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "3", "11", "12", "13"]);
}
#[test]
fn repeat_in_for() {
    let out = compile_and_run(
        "\
program RepeatInFor;
begin
  for I: integer := 1 to 2 do
  begin
    mutable var J: integer := 0;
    repeat
      Std.Console.WriteLn(I * 10 + J);
      J := J + 1
    until J = 3
  end
end.",
    );
    assert_eq!(out.lines, vec!["10", "11", "12", "20", "21", "22"]);
}
#[test]
fn while_in_repeat() {
    let out = compile_and_run(
        "\
program WhileInRepeat;
begin
  mutable var I: integer := 0;
  repeat
    mutable var J: integer := 0;
    while J < 2 do
    begin
      Std.Console.WriteLn(I * 10 + J);
      J := J + 1
    end;
    I := I + 1
  until I = 2
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "10", "11"]);
}
#[test]
fn repeat_in_while() {
    let out = compile_and_run(
        "\
program RepeatInWhile;
begin
  mutable var I: integer := 0;
  while I < 2 do
  begin
    mutable var J: integer := 0;
    repeat
      Std.Console.WriteLn(I * 10 + J);
      J := J + 1
    until J = 3;
    I := I + 1
  end
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "2", "10", "11", "12"]);
}
#[test]
fn for_in_while() {
    let out = compile_and_run(
        "\
program ForInWhile;
begin
  mutable var I: integer := 0;
  while I < 2 do
  begin
    for J: integer := 1 to 3 do
      Std.Console.WriteLn(I * 10 + J);
    I := I + 1
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "3", "11", "12", "13"]);
}
#[test]
fn while_in_for() {
    let out = compile_and_run(
        "\
program WhileInFor;
begin
  for I: integer := 1 to 2 do
  begin
    mutable var J: integer := 0;
    while J < 3 do
    begin
      Std.Console.WriteLn(I * 10 + J);
      J := J + 1
    end
  end
end.",
    );
    assert_eq!(out.lines, vec!["10", "11", "12", "20", "21", "22"]);
}

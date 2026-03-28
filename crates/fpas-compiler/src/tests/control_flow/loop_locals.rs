use super::*;
// repeat: variables declared inside body are scoped per iteration.
#[test]
fn repeat_var_in_body() {
    let out = compile_and_run(
        "\
program RepeatVar;
begin
  mutable var I: integer := 0;
  repeat
    var X: integer := I * 10;
    Std.Console.WriteLn(X);
    I := I + 1
  until I = 3
end.",
    );
    assert_eq!(out.lines, vec!["0", "10", "20"]);
}
#[test]
fn repeat_multiple_vars_in_body() {
    let out = compile_and_run(
        "\
program RepeatMultiVar;
begin
  mutable var I: integer := 1;
  repeat
    var A: integer := I;
    var B: integer := I * 2;
    Std.Console.WriteLn(A + B);
    I := I + 1
  until I > 3
end.",
    );
    assert_eq!(out.lines, vec!["3", "6", "9"]);
}
#[test]
fn repeat_with_break() {
    let out = compile_and_run(
        "\
program RepeatBreak;
begin
  mutable var I: integer := 0;
  repeat
    if I = 2 then
      break;
    Std.Console.WriteLn(I);
    I := I + 1
  until I = 10
end.",
    );
    assert_eq!(out.lines, vec!["0", "1"]);
}
#[test]
fn repeat_condition_uses_body_var() {
    let out = compile_and_run(
        "\
program RepeatCondVar;
begin
  mutable var N: integer := 0;
  repeat
    N := N + 1;
    var Doubled: integer := N * 2;
    Std.Console.WriteLn(Doubled)
  until N >= 4
end.",
    );
    assert_eq!(out.lines, vec!["2", "4", "6", "8"]);
}
// while: variables declared inside body.
#[test]
fn while_var_in_body() {
    let out = compile_and_run(
        "\
program WhileVar;
begin
  mutable var I: integer := 0;
  while I < 3 do
  begin
    var Msg: integer := I + 100;
    Std.Console.WriteLn(Msg);
    I := I + 1
  end
end.",
    );
    assert_eq!(out.lines, vec!["100", "101", "102"]);
}
// for: variable scoping.
#[test]
fn for_var_in_body() {
    let out = compile_and_run(
        "\
program ForVar;
begin
  for I: integer := 1 to 3 do
  begin
    var Sq: integer := I * I;
    Std.Console.WriteLn(Sq)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "4", "9"]);
}
// loops with variables + if/else inside.
#[test]
fn repeat_with_var_and_if() {
    let out = compile_and_run(
        "\
program RepeatVarIf;
begin
  mutable var I: integer := 0;
  repeat
    mutable var Label: string := 'odd';
    if I mod 2 = 0 then
      Label := 'even';
    Std.Console.WriteLn(Label);
    I := I + 1
  until I = 4
end.",
    );
    assert_eq!(out.lines, vec!["even", "odd", "even", "odd"]);
}
#[test]
fn nested_repeat_with_vars_and_break() {
    let out = compile_and_run(
        "\
program RepeatVarBreak;
begin
  mutable var I: integer := 0;
  repeat
    var Prefix: integer := I * 100;
    mutable var J: integer := 0;
    repeat
      var Val: integer := Prefix + J;
      if J = 2 then
        break;
      Std.Console.WriteLn(Val);
      J := J + 1
    until J = 10;
    I := I + 1
  until I = 3
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "100", "101", "200", "201"]);
}
// loop with mutable var reset per iteration.
#[test]
fn repeat_mutable_var_per_iteration() {
    let out = compile_and_run(
        "\
program RepeatMutVar;
begin
  mutable var Sum: integer := 0;
  mutable var I: integer := 1;
  repeat
    mutable var Accum: integer := 0;
    Accum := Accum + I;
    Sum := Sum + Accum;
    I := I + 1
  until I > 5;
  Std.Console.WriteLn(Sum)
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}

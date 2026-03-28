use super::*;

#[test]
fn repeat_break_immediately() {
    let out = compile_and_run(
        "\
program RepeatBreakImm;
begin
  mutable var Reached: integer := 0;
  repeat
    break;
    Reached := Reached + 1
  until false;
  Std.Console.WriteLn(Reached)
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn repeat_break_is_only_exit() {
    let out = compile_and_run(
        "\
program RepeatBreakOnly;
begin
  mutable var I: integer := 0;
  repeat
    I := I + 1;
    if I = 5 then
      break
  until false;
  Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn repeat_continue_evaluates_until() {
    let out = compile_and_run(
        "\
program RepContUntil;
begin
  mutable var I: integer := 0;
  repeat
    I := I + 1;
    if I < 4 then
      continue;
    Std.Console.WriteLn(I)
  until I >= 5
end.",
    );
    assert_eq!(out.lines, vec!["4", "5"]);
}

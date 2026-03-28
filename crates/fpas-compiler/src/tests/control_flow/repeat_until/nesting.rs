use super::*;

#[test]
fn repeat_break_only_inner() {
    let out = compile_and_run(
        "\
program RepeatBreakInner;
begin
  mutable var Outer: integer := 0;
  repeat
    mutable var Inner: integer := 0;
    repeat
      if Inner = 1 then
        break;
      Std.Console.WriteLn(Inner);
      Inner := Inner + 1
    until Inner = 10;
    Outer := Outer + 1
  until Outer = 3
end.",
    );
    assert_eq!(out.lines, vec!["0", "0", "0"]);
}

#[test]
fn repeat_continue_only_inner() {
    let out = compile_and_run(
        "\
program RepeatContinueInner;
begin
  mutable var Outer: integer := 0;
  repeat
    mutable var Inner: integer := 0;
    repeat
      Inner := Inner + 1;
      if Inner mod 2 = 0 then
        continue;
      Std.Console.WriteLn(Inner)
    until Inner >= 4;
    Outer := Outer + 1
  until Outer = 2
end.",
    );
    assert_eq!(out.lines, vec!["1", "3", "1", "3"]);
}

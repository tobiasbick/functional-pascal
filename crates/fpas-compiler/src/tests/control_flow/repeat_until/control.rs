use super::*;

#[test]
fn repeat_continue_skips_rest_of_body() {
    let out = compile_and_run(
        "\
program RepeatContinue;
begin
  mutable var I: integer := 0;
  repeat
    I := I + 1;
    if I mod 2 = 0 then
      continue;
    Std.Console.WriteLn(I)
  until I >= 6
end.",
    );
    assert_eq!(out.lines, vec!["1", "3", "5"]);
}

#[test]
fn repeat_continue_all_skipped() {
    let out = compile_and_run(
        "\
program RepeatContinueAll;
begin
  mutable var I: integer := 0;
  repeat
    I := I + 1;
    continue;
    Std.Console.WriteLn('never')
  until I = 3;
  Std.Console.WriteLn('done')
end.",
    );
    assert_eq!(out.lines, vec!["done"]);
}

#[test]
fn repeat_break_and_continue() {
    let out = compile_and_run(
        "\
program RepeatBreakContinue;
begin
  mutable var I: integer := 0;
  repeat
    I := I + 1;
    if I mod 2 = 0 then
      continue;
    if I > 7 then
      break;
    Std.Console.WriteLn(I)
  until false
end.",
    );
    assert_eq!(out.lines, vec!["1", "3", "5", "7"]);
}

#[test]
fn repeat_break_then_code_after() {
    let out = compile_and_run(
        "\
program RepeatBreakAfter;
begin
  mutable var I: integer := 0;
  repeat
    I := I + 1;
    if I = 2 then
      break
  until false;
  Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["2"]);
}

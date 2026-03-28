use super::*;

#[test]
fn repeat_executes_at_least_once() {
    let out = compile_and_run(
        "\
program RepeatOnce;
begin
  repeat
    Std.Console.WriteLn('once')
  until true
end.",
    );
    assert_eq!(out.lines, vec!["once"]);
}

#[test]
fn repeat_single_statement_body() {
    let out = compile_and_run(
        "\
program RepeatSingle;
begin
  mutable var I: integer := 0;
  repeat
    I := I + 1
  until I = 5;
  Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn repeat_condition_true_on_first_check() {
    let out = compile_and_run(
        "\
program RepeatImmediate;
begin
  mutable var I: integer := 10;
  repeat
    Std.Console.WriteLn(I);
    I := I + 1
  until I > 5
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

#[test]
fn repeat_code_after_loop() {
    let out = compile_and_run(
        "\
program RepeatAfter;
begin
  mutable var I: integer := 0;
  repeat
    I := I + 1
  until I = 3;
  Std.Console.WriteLn('after');
  Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["after", "3"]);
}

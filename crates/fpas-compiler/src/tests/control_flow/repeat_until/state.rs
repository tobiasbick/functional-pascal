use super::*;

#[test]
fn repeat_accumulator() {
    let out = compile_and_run(
        "\
program RepeatAccum;
begin
  mutable var Sum: integer := 0;
  mutable var I: integer := 1;
  repeat
    Sum := Sum + I;
    I := I + 1
  until I > 5;
  Std.Console.WriteLn(Sum)
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}

#[test]
fn repeat_string_accumulation() {
    let out = compile_and_run(
        "\
program RepeatStrAccum;
begin
  mutable var S: string := '';
  mutable var I: integer := 0;
  repeat
    S := S + 'x';
    I := I + 1
  until I = 3;
  Std.Console.WriteLn(S)
end.",
    );
    assert_eq!(out.lines, vec!["xxx"]);
}

#[test]
fn repeat_multi_statement_without_begin_end() {
    let out = compile_and_run(
        "\
program RepeatMulti;
begin
  mutable var A: integer := 0;
  mutable var B: integer := 100;
  repeat
    A := A + 1;
    B := B - 10;
    Std.Console.WriteLn(A);
    Std.Console.WriteLn(B)
  until A = 3
end.",
    );
    assert_eq!(out.lines, vec!["1", "90", "2", "80", "3", "70"]);
}

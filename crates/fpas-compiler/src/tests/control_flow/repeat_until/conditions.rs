use super::*;

#[test]
fn repeat_and_condition() {
    let out = compile_and_run(
        "\
program RepeatAnd;
begin
  mutable var I: integer := 0;
  repeat
    Std.Console.WriteLn(I);
    I := I + 1
  until (I > 2) and (I mod 2 = 1)
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "2"]);
}

#[test]
fn repeat_or_condition() {
    let out = compile_and_run(
        "\
program RepeatOr;
begin
  mutable var I: integer := 0;
  repeat
    Std.Console.WriteLn(I);
    I := I + 1
  until (I = 3) or (I = 5)
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "2"]);
}

#[test]
fn repeat_not_condition() {
    let out = compile_and_run(
        "\
program RepeatNot;
begin
  mutable var Running: boolean := true;
  mutable var I: integer := 0;
  repeat
    Std.Console.WriteLn(I);
    I := I + 1;
    if I = 3 then
      Running := false
  until not Running
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "2"]);
}

#[test]
fn repeat_greater_than_condition() {
    let out = compile_and_run(
        "\
program RepeatGt;
begin
  mutable var I: integer := 0;
  repeat
    Std.Console.WriteLn(I);
    I := I + 1
  until I > 2
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "2"]);
}

#[test]
fn repeat_less_than_condition() {
    let out = compile_and_run(
        "\
program RepeatLt;
begin
  mutable var I: integer := 10;
  repeat
    Std.Console.WriteLn(I);
    I := I - 3
  until I < 3
end.",
    );
    assert_eq!(out.lines, vec!["10", "7", "4"]);
}

#[test]
fn repeat_not_equal_condition() {
    let out = compile_and_run(
        "\
program RepeatNeq;
begin
  mutable var I: integer := 0;
  repeat
    Std.Console.WriteLn(I);
    I := I + 1
  until I <> 0
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn repeat_boolean_variable_condition() {
    let out = compile_and_run(
        "\
program RepeatBoolVar;
begin
  mutable var Done: boolean := false;
  mutable var I: integer := 0;
  repeat
    Std.Console.WriteLn(I);
    I := I + 1;
    if I = 3 then
      Done := true
  until Done
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "2"]);
}

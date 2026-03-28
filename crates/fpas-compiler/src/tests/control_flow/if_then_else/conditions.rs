use super::*;

#[test]
fn if_not_condition() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: boolean := false;
  if not X then
    Std.Console.WriteLn('negated')
end.",
    );
    assert_eq!(out.lines, vec!["negated"]);
}

#[test]
fn if_and_condition() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 5;
  if (X > 0) and (X < 10) then
    Std.Console.WriteLn('in range')
  else
    Std.Console.WriteLn('out of range')
end.",
    );
    assert_eq!(out.lines, vec!["in range"]);
}

#[test]
fn if_or_condition() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := -1;
  if (X > 10) or (X < 0) then
    Std.Console.WriteLn('extreme')
  else
    Std.Console.WriteLn('moderate')
end.",
    );
    assert_eq!(out.lines, vec!["extreme"]);
}

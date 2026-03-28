use super::*;

#[test]
fn if_greater_than() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 5;
  if X > 0 then
    Std.Console.WriteLn('positive')
  else
    Std.Console.WriteLn('non-positive')
end.",
    );
    assert_eq!(out.lines, vec!["positive"]);
}

#[test]
fn if_equal() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 0;
  if X = 0 then
    Std.Console.WriteLn('zero')
  else
    Std.Console.WriteLn('not zero')
end.",
    );
    assert_eq!(out.lines, vec!["zero"]);
}

#[test]
fn if_less_than() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := -1;
  if X < 0 then
    Std.Console.WriteLn('negative')
  else
    Std.Console.WriteLn('non-negative')
end.",
    );
    assert_eq!(out.lines, vec!["negative"]);
}

use super::*;

#[test]
fn else_if_first_branch() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 5;
  if X > 0 then
    Std.Console.WriteLn('positive')
  else if X = 0 then
    Std.Console.WriteLn('zero')
  else
    Std.Console.WriteLn('negative')
end.",
    );
    assert_eq!(out.lines, vec!["positive"]);
}

#[test]
fn else_if_middle_branch() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 0;
  if X > 0 then
    Std.Console.WriteLn('positive')
  else if X = 0 then
    Std.Console.WriteLn('zero')
  else
    Std.Console.WriteLn('negative')
end.",
    );
    assert_eq!(out.lines, vec!["zero"]);
}

#[test]
fn else_if_last_branch() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := -3;
  if X > 0 then
    Std.Console.WriteLn('positive')
  else if X = 0 then
    Std.Console.WriteLn('zero')
  else
    Std.Console.WriteLn('negative')
end.",
    );
    assert_eq!(out.lines, vec!["negative"]);
}

#[test]
fn deeply_chained_else_if() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 3;
  if X = 1 then
    Std.Console.WriteLn('one')
  else if X = 2 then
    Std.Console.WriteLn('two')
  else if X = 3 then
    Std.Console.WriteLn('three')
  else if X = 4 then
    Std.Console.WriteLn('four')
  else
    Std.Console.WriteLn('other')
end.",
    );
    assert_eq!(out.lines, vec!["three"]);
}

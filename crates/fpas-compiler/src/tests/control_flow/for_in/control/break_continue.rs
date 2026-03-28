use super::*;

#[test]
fn for_in_with_break() {
    let out = compile_and_run(
        "\
program ForInBreak;
begin
  var Arr: array of integer := [1, 2, 3, 4, 5];
  for X: integer in Arr do
  begin
    if X = 3 then break;
    Std.Console.WriteLn(X)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "2"]);
}

#[test]
fn for_in_with_continue() {
    let out = compile_and_run(
        "\
program ForInContinue;
begin
  var Arr: array of integer := [1, 2, 3, 4, 5];
  for X: integer in Arr do
  begin
    if X mod 2 = 0 then continue;
    Std.Console.WriteLn(X)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "3", "5"]);
}

#[test]
fn for_in_break_first_element() {
    let out = compile_and_run(
        "\
program ForInBreakFirst;
begin
  var Arr: array of integer := [1, 2, 3];
  mutable var Count: integer := 0;
  for X: integer in Arr do
  begin
    break;
    Count := Count + 1
  end;
  Std.Console.WriteLn(Count)
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn for_in_continue_all() {
    let out = compile_and_run(
        "\
program ForInContinueAll;
begin
  var Arr: array of integer := [1, 2, 3];
  mutable var Reached: integer := 0;
  for X: integer in Arr do
  begin
    continue;
    Reached := Reached + 1
  end;
  Std.Console.WriteLn(Reached)
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn for_in_break_and_continue() {
    let out = compile_and_run(
        "\
program ForInBC;
begin
  var Arr: array of integer := [1, 2, 3, 4, 5, 6, 7, 8];
  for X: integer in Arr do
  begin
    if X mod 2 = 0 then continue;
    if X > 5 then break;
    Std.Console.WriteLn(X)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "3", "5"]);
}

#[test]
fn for_in_continue_then_break_on_same_element() {
    let out = compile_and_run(
        "\
program ForInContBreak;
begin
  var Arr: array of integer := [2, 4, 5, 6, 7];
  for X: integer in Arr do
  begin
    if X mod 2 = 0 then
      continue;
    if X = 7 then
      break;
    Std.Console.WriteLn(X)
  end
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

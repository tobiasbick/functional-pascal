use super::*;

#[test]
fn for_in_break_and_continue_combined() {
    let out = compile_and_run(
        "\
program ForInBrkCont;
begin
  var Arr: array of integer := [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
  for X: integer in Arr do
  begin
    if X mod 2 = 0 then
      continue;
    if X > 7 then
      break;
    Std.Console.WriteLn(X)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "3", "5", "7"]);
}

#[test]
fn for_in_break_first_element() {
    let out = compile_and_run(
        "\
program ForInBrkFirst;
begin
  var Arr: array of integer := [10, 20, 30];
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
fn for_in_continue_all_elements() {
    let out = compile_and_run(
        "\
program ForInContAll;
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

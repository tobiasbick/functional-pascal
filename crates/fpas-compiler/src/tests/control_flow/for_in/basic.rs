use super::*;

#[test]
fn for_in_array() {
    let out = compile_and_run(
        "\
program ForInTest;
begin
  var Arr: array of integer := [10, 20, 30];
  for X: integer in Arr do
    Std.Console.WriteLn(X)
end.",
    );
    assert_eq!(out.lines, vec!["10", "20", "30"]);
}

#[test]
fn for_in_empty_array() {
    let out = compile_and_run(
        "\
program ForInEmpty;
begin
  var Arr: array of integer := [];
  mutable var Count: integer := 0;
  for X: integer in Arr do
    Count := Count + 1;
  Std.Console.WriteLn(Count)
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn for_in_single_element() {
    let out = compile_and_run(
        "\
program ForInSingle;
begin
  var Arr: array of integer := [42];
  for X: integer in Arr do
    Std.Console.WriteLn(X)
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn for_in_inline_array_literal() {
    let out = compile_and_run(
        "\
program ForInLiteral;
begin
  for X: integer in [10, 20, 30] do
    Std.Console.WriteLn(X)
end.",
    );
    assert_eq!(out.lines, vec!["10", "20", "30"]);
}

#[test]
fn for_in_inline_empty_literal() {
    let out = compile_and_run(
        "\
program ForInEmptyLiteral;
begin
  mutable var Count: integer := 0;
  for X: integer in [] do
    Count := Count + 1;
  Std.Console.WriteLn(Count)
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn for_in_large_array() {
    let out = compile_and_run(
        "\
program ForInLarge;
begin
  var Arr: array of integer := [1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
  mutable var Sum: integer := 0;
  for X: integer in Arr do
    Sum := Sum + X;
  Std.Console.WriteLn(Sum)
end.",
    );
    assert_eq!(out.lines, vec!["210"]);
}

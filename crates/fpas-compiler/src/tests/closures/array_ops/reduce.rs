use super::*;

#[test]
fn array_reduce_sum() {
    let out = compile_and_run(
        "\
program ReduceTest;
uses Std.Console, Std.Array;
begin
  var Nums: array of integer := [1, 2, 3, 4, 5];
  var Sum: integer := Reduce(Nums, 0,
    function(Acc: integer; V: integer): integer
    begin
      return Acc + V
    end);
  WriteLn(Sum)
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}

#[test]
fn reduce_empty_array_returns_init() {
    let out = compile_and_run(
        "\
program ReduceEmpty;
uses Std.Console, Std.Array;
begin
  var Empty: array of integer := [];
  var Sum: integer := Reduce(Empty, 42,
    function(Acc: integer; V: integer): integer begin return Acc + V end);
  WriteLn(Sum)
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn reduce_single_element() {
    let out = compile_and_run(
        "\
program ReduceSingle;
uses Std.Console, Std.Array;
begin
  var One: array of integer := [7];
  var Res: integer := Reduce(One, 100,
    function(Acc: integer; V: integer): integer begin return Acc + V end);
  WriteLn(Res)
end.",
    );
    assert_eq!(out.lines, vec!["107"]);
}

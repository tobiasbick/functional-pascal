use super::*;

#[test]
fn map_then_filter() {
    let out = compile_and_run(
        "\
program ChainTest;
uses Std.Console, Std.Array;
begin
  var Nums: array of integer := [1, 2, 3, 4, 5];
  var Doubled: array of integer := Map(Nums,
    function(X: integer): integer begin return X * 2 end);
  var Big: array of integer := Filter(Doubled,
    function(X: integer): boolean begin return X > 5 end);
  for V: integer in Big do
    Write(V);
  WriteLn('')
end.",
    );
    assert_eq!(out.lines, vec!["6810"]);
}

#[test]
fn map_filter_reduce_chained() {
    let out = compile_and_run(
        "\
program FullChain;
uses Std.Console, Std.Array;
begin
  var Nums: array of integer := [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
  { Square, keep > 20, sum }
  var Squared: array of integer := Map(Nums,
    function(X: integer): integer begin return X * X end);
  var Big: array of integer := Filter(Squared,
    function(X: integer): boolean begin return X > 20 end);
  var Total: integer := Reduce(Big, 0,
    function(Acc: integer; V: integer): integer begin return Acc + V end);
  WriteLn(Total)
end.",
    );
    assert_eq!(out.lines, vec!["355"]);
}

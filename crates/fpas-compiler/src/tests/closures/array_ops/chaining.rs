use super::*;

#[test]
fn map_then_filter() {
    let out = compile_and_run(
        "\
program ChainTest;
uses Std.Console, Std.Array;
function Double(X: integer): integer;
begin
  return X * 2
end;
function GreaterThanFive(X: integer): boolean;
begin
  return X > 5
end;
begin
  var Nums: array of integer := [1, 2, 3, 4, 5];
  var Doubled: array of integer := Map(Nums, Double);
  var Big: array of integer := Filter(Doubled, GreaterThanFive);
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
function Square(X: integer): integer;
begin
  return X * X
end;
function GreaterThanTwenty(X: integer): boolean;
begin
  return X > 20
end;
function AddAcc(Acc: integer; V: integer): integer;
begin
  return Acc + V
end;
begin
  var Nums: array of integer := [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
  var Squared: array of integer := Map(Nums, Square);
  var Big: array of integer := Filter(Squared, GreaterThanTwenty);
  var Total: integer := Reduce(Big, 0, AddAcc);
  WriteLn(Total)
end.",
    );
    assert_eq!(out.lines, vec!["355"]);
}

use super::*;

#[test]
fn array_filter_with_named_function() {
    let out = compile_and_run(
        "\
program FilterTest;
uses Std.Console, Std.Array;
function IsEven(X: integer): boolean;
begin
  return X mod 2 = 0
end;
begin
  var Nums: array of integer := [1, 2, 3, 4, 5, 6];
  var Evens: array of integer := Filter(Nums, IsEven);
  for V: integer in Evens do
    Write(V);
  WriteLn('')
end.",
    );
    assert_eq!(out.lines, vec!["246"]);
}

#[test]
fn filter_empty_array() {
    let out = compile_and_run(
        "\
program FilterEmpty;
uses Std.Console, Std.Array;
function AlwaysTrue(X: integer): boolean;
begin
  return true
end;
begin
  var Empty: array of integer := [];
  var Res: array of integer := Filter(Empty, AlwaysTrue);
  WriteLn(Length(Res))
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn filter_no_match_returns_empty() {
    let out = compile_and_run(
        "\
program FilterNone;
uses Std.Console, Std.Array;
function GreaterThanHundred(X: integer): boolean;
begin
  return X > 100
end;
begin
  var Nums: array of integer := [1, 2, 3];
  var Res: array of integer := Filter(Nums, GreaterThanHundred);
  WriteLn(Length(Res))
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

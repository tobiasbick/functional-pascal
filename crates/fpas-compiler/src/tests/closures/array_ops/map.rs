use super::*;

#[test]
fn array_map_with_lambda() {
    let out = compile_and_run(
        "\
program MapTest;
uses Std.Console, Std.Array;
begin
  var Numbers: array of integer := [1, 2, 3, 4, 5];
  var Doubled: array of integer := Map(Numbers,
    function(X: integer): integer
    begin
      return X * 2
    end);
  for V: integer in Doubled do
    Write(V);
  WriteLn('')
end.",
    );
    assert_eq!(out.lines, vec!["246810"]);
}

#[test]
fn array_map_with_named_function() {
    let out = compile_and_run(
        "\
program MapNamed;
uses Std.Console, Std.Array;
function Square(X: integer): integer;
begin
  return X * X
end;
begin
  var Nums: array of integer := [1, 2, 3];
  var Squared: array of integer := Map(Nums, Square);
  for V: integer in Squared do
    Write(V);
  WriteLn('')
end.",
    );
    assert_eq!(out.lines, vec!["149"]);
}

#[test]
fn map_empty_array() {
    let out = compile_and_run(
        "\
program MapEmpty;
uses Std.Console, Std.Array;
begin
  var Empty: array of integer := [];
  var Res: array of integer := Map(Empty,
    function(X: integer): integer begin return X * 2 end);
  WriteLn(Length(Res))
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn map_with_closure_capturing_variable() {
    let out = compile_and_run(
        "\
program MapClosure;
uses Std.Console, Std.Array;
function MakeMapper(Factor: integer): function(X: integer): integer;
begin
  return function(X: integer): integer begin return X * Factor end
end;
begin
  var Nums: array of integer := [1, 2, 3];
  var Tripled: array of integer := Map(Nums, MakeMapper(3));
  for V: integer in Tripled do
    Write(V, ' ');
  WriteLn('')
end.",
    );
    assert_eq!(out.lines, vec!["3 6 9 "]);
}

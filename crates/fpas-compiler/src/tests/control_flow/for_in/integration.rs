use super::*;

#[test]
fn for_in_accumulator() {
    let out = compile_and_run(
        "\
program ForInAccum;
begin
  var Arr: array of integer := [10, 20, 30, 40];
  mutable var Sum: integer := 0;
  for X: integer in Arr do
    Sum := Sum + X;
  Std.Console.WriteLn(Sum)
end.",
    );
    assert_eq!(out.lines, vec!["100"]);
}

#[test]
fn for_in_inside_function() {
    let out = compile_and_run(
        "\
program ForInFunc;

function SumArray(Arr: array of integer): integer;
begin
  mutable var Total: integer := 0;
  for X: integer in Arr do
    Total := Total + X;
  return Total
end;

begin
  Std.Console.WriteLn(SumArray([4, 5, 6]))
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}

#[test]
fn sequential_for_in_loops() {
    let out = compile_and_run(
        "\
program ForInSeq;
begin
  var A: array of integer := [1, 2];
  var B: array of string := ['a', 'b'];
  for X: integer in A do
    Std.Console.WriteLn(X);
  for S: string in B do
    Std.Console.WriteLn(S)
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "a", "b"]);
}

#[test]
fn for_in_body_uses_loop_var_in_expression() {
    let out = compile_and_run(
        "\
program ForInExpr;
begin
  var Arr: array of integer := [3, 7, 11];
  for X: integer in Arr do
    Std.Console.WriteLn(X * 2 + 1)
end.",
    );
    assert_eq!(out.lines, vec!["7", "15", "23"]);
}

#[test]
fn for_in_with_nested_if_else() {
    let out = compile_and_run(
        "\
program ForInIfElse;
begin
  var Arr: array of integer := [1, 2, 3, 4, 5];
  for X: integer in Arr do
  begin
    if X mod 2 = 0 then
      Std.Console.WriteLn('even')
    else
      Std.Console.WriteLn('odd')
  end
end.",
    );
    assert_eq!(out.lines, vec!["odd", "even", "odd", "even", "odd"]);
}

#[test]
fn for_in_loop_var_shadows_outer() {
    let out = compile_and_run(
        "\
program ForInShadow;
begin
  var X: integer := 999;
  var Arr: array of integer := [1, 2, 3];
  for X: integer in Arr do
    Std.Console.WriteLn(X);
  Std.Console.WriteLn(X)
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "3", "999"]);
}

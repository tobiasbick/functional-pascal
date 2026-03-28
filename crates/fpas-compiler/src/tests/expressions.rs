use super::*;

#[test]
fn boolean_logic() {
    let out = compile_and_run(
        "\
program BoolTest;
begin
  Std.Console.WriteLn(true and false);
  Std.Console.WriteLn(true or false);
  Std.Console.WriteLn(not false)
end.",
    );
    assert_eq!(out.lines, vec!["false", "true", "true"]);
}

#[test]
fn string_output() {
    let out = compile_and_run(
        "\
program StrTest;
begin
  Std.Console.WriteLn('hello')
end.",
    );
    assert_eq!(out.lines, vec!["hello"]);
}

#[test]
fn nested_calls() {
    let out = compile_and_run(
        "\
program Nested;

function Add(a: integer; b: integer): integer;
begin
  return a + b
end;

function Mul(a: integer; b: integer): integer;
begin
  return a * b
end;

begin
  Std.Console.WriteLn(Add(Mul(2, 3), Mul(4, 5)))
end.",
    );
    assert_eq!(out.lines, vec!["26"]);
}

#[test]
fn break_in_while() {
    let out = compile_and_run(
        "\
program BreakTest;
begin
  mutable var I: integer := 0;
  while true do
  begin
    if I = 3 then
      break;
    Std.Console.WriteLn(I);
    I := I + 1
  end
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "2"]);
}

#[test]
fn negation() {
    let out = compile_and_run(
        "\
program NegTest;
begin
  Std.Console.WriteLn(-42)
end.",
    );
    assert_eq!(out.lines, vec!["-42"]);
}

#[test]
fn const_decl() {
    let out = compile_and_run(
        "\
program ConstTest;
const
  Pi: integer := 314;
begin
  Std.Console.WriteLn(Pi)
end.",
    );
    assert_eq!(out.lines, vec!["314"]);
}

#[test]
fn array_literal() {
    let out = compile_and_run(
        "\
program ArrTest;
begin
  var Arr: array of integer := [10, 20, 30];
  Std.Console.WriteLn(Arr)
end.",
    );
    assert_eq!(out.lines, vec!["[10, 20, 30]"]);
}

#[test]
fn array_index_get() {
    let out = compile_and_run(
        "\
program ArrIdx;
begin
  var A: array of integer := [10, 20, 30];
  Std.Console.WriteLn(A[0]);
  Std.Console.WriteLn(A[1]);
  Std.Console.WriteLn(A[2])
end.",
    );
    assert_eq!(out.lines, vec!["10", "20", "30"]);
}

#[test]
fn array_index_set() {
    let out = compile_and_run(
        "\
program ArrSet;
begin
  mutable var A: array of integer := [1, 2, 3];
  A[0] := 99;
  A[2] := 77;
  Std.Console.WriteLn(A[0]);
  Std.Console.WriteLn(A[1]);
  Std.Console.WriteLn(A[2])
end.",
    );
    assert_eq!(out.lines, vec!["99", "2", "77"]);
}

#[test]
fn array_index_with_expression() {
    let out = compile_and_run(
        "\
program ArrExprIdx;
begin
  var A: array of integer := [10, 20, 30, 40, 50];
  var I: integer := 2;
  Std.Console.WriteLn(A[I]);
  Std.Console.WriteLn(A[I + 1])
end.",
    );
    assert_eq!(out.lines, vec!["30", "40"]);
}

#[test]
fn array_in_loop() {
    let out = compile_and_run(
        "\
program ArrLoop;
begin
  var A: array of integer := [100, 200, 300];
  for I: integer := 0 to 2 do
    Std.Console.WriteLn(A[I])
end.",
    );
    assert_eq!(out.lines, vec!["100", "200", "300"]);
}

use super::*;

#[test]
fn fibonacci() {
    check_ok(
        "\
program Fib;

function Fibonacci(N: integer): integer;
begin
  if N <= 1 then
    return N
  else
    return Fibonacci(N - 1) + Fibonacci(N - 2)
end;

begin
  return
end.",
    );
}

#[test]
fn calculator() {
    check_ok(
        "\
program Calculator;

type Op = enum
  OpAdd;
  OpSub;
  OpMul;
  OpDiv;
end;

function Calculate(A: integer; B: integer; Operation: Op): integer;
begin
  case Operation of
    OpAdd: return A + B;
    OpSub: return A - B;
    OpMul: return A * B;
    OpDiv: return A div B
  end
end;

begin
  var Answer: integer := Calculate(10, 3, OpAdd)
end.",
    );
}

#[test]
fn record_usage() {
    check_ok(
        "\
program Geometry;

type Point = record
  X: real;
  Y: real;
end;

begin
  var P: Point := record X := 1.0; Y := 2.0; end
end.",
    );
}

#[test]
fn nested_loops_with_break_continue() {
    check_ok(
        "\
program T;
begin
  for I: integer := 0 to 9 do
    for J: integer := 0 to 9 do
      begin
        if I = J then continue;
        if I + J > 10 then break
      end
end.",
    );
}

#[test]
fn forward_declaration() {
    check_ok(
        "\
program T;

function IsEven(N: integer): boolean; forward;
function IsOdd(N: integer): boolean; forward;

function IsEven(N: integer): boolean;
begin
  if N = 0 then return true
  else return IsOdd(N - 1)
end;

function IsOdd(N: integer): boolean;
begin
  if N = 0 then return false
  else return IsEven(N - 1)
end;

begin
  return
end.",
    );
}

#[test]
fn immutable_assignment_error() {
    check_errors(
        "\
program T;
var X: integer := 42;
begin
  X := 100
end.",
    );
}

#[test]
fn mixed_errors() {
    let errs = check_errors(
        "\
program T;
begin
  break;
  var X: integer := true
end.",
    );
    assert!(errs.len() >= 2);
}

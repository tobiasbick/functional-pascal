use super::parse_ok;
use crate::ast::*;

#[test]
fn hello_world() {
    let p = parse_ok(
        "\
program Hello;
uses
  Std.Console;
begin
  Std.Console.WriteLn('Hello, World!')
end.",
    );
    assert_eq!(p.name, "Hello");
    assert_eq!(p.uses.len(), 1);
    assert_eq!(p.uses[0].parts, vec!["Std", "Console"]);
    assert_eq!(p.body.len(), 1);
    assert!(matches!(&p.body[0], Stmt::Call { .. }));
}

#[test]
fn fibonacci() {
    let p = parse_ok(
        "\
program Fib;
uses Std.Console;

function Fibonacci(N: integer): integer;
begin
  if N <= 1 then
    return N
  else
    return Fibonacci(N - 1) + Fibonacci(N - 2)
end;

begin
  Std.Console.WriteLn(Fibonacci(10))
end.",
    );
    assert_eq!(p.name, "Fib");
    assert_eq!(p.declarations.len(), 1);
    assert!(matches!(&p.declarations[0], Decl::Function(_)));
}

#[test]
fn full_program() {
    let p = parse_ok(
        "\
program Calculator;
uses Std.Console;

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
  var Answer: integer := Calculate(10, 3, OpAdd);
  Std.Console.WriteLn(Answer)
end.",
    );
    assert_eq!(p.name, "Calculator");
    assert_eq!(p.declarations.len(), 2); // type + function
    assert_eq!(p.body.len(), 2); // var + call
}

#[test]
fn record_creation_and_access() {
    let p = parse_ok(
        "\
program Geometry;

type Point = record
  X: real;
  Y: real;
end;

begin
  var P: Point := record X := 1.0; Y := 2.0; end;
  var Sum: real := P.X + P.Y
end.",
    );
    assert_eq!(p.declarations.len(), 1);
    assert_eq!(p.body.len(), 2);
}

#[test]
fn nested_loops() {
    let p = parse_ok(
        "\
program T;
begin
  for I: integer := 0 to 9 do
    for J: integer := 0 to 9 do
      begin
        var X: integer := I * 10 + J;
        if X mod 2 = 0 then
          continue
      end
end.",
    );
    assert_eq!(p.body.len(), 1);
    match &p.body[0] {
        Stmt::For { body, .. } => {
            assert!(matches!(body.as_ref(), Stmt::For { .. }));
        }
        _ => panic!("expected nested For"),
    }
}

#[test]
fn repeat_with_break() {
    let p = parse_ok(
        "\
program T;
begin
  mutable var X: integer := 0;
  repeat
    X := X + 1;
    if X = 10 then break
  until X = 100
end.",
    );
    assert_eq!(p.body.len(), 2);
}

#[test]
fn nested_mutual_recursion_even_odd() {
    let p = parse_ok(
        "\
program T;

function IsEven(N: integer): boolean;
  function IsOdd(X: integer): boolean;
  begin
    if X = 0 then return false
    else return IsEven(X - 1)
  end;
begin
  if N = 0 then return true
  else return IsOdd(N - 1)
end;

begin
  return
end.",
    );
    assert_eq!(p.declarations.len(), 1);
    match &p.declarations[0] {
        Decl::Function(f) => match &f.body {
            FuncBody::Block { nested, .. } => assert_eq!(nested.len(), 1),
        },
        _ => panic!("expected Function"),
    }
}

#[test]
fn array_operations() {
    let p = parse_ok(
        "\
program T;
begin
  var Xs: array of integer := [1, 2, 3, 4, 5];
  var First: integer := Xs[0];
  var Last: integer := Xs[4]
end.",
    );
    assert_eq!(p.body.len(), 3);
}

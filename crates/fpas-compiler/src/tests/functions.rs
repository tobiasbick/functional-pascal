/// Tests for basic function and procedure declarations.
///
/// **Documentation:** [docs/pascal/04-functions.md](docs/pascal/04-functions.md)
use super::*;

// ═══════════════════════════════════════════════════════════════
// BASIC FUNCTIONS
// ═══════════════════════════════════════════════════════════════

#[test]
fn simple_function() {
    let out = compile_and_run(
        "\
program FuncTest;

function Double(x: integer): integer;
begin
  return x * 2
end;

begin
  Std.Console.WriteLn(Double(21))
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn function_returning_string() {
    let out = compile_and_run(
        "\
program FuncStr;
uses Std.Console;

function Greet(Name: string): string;
begin
  return 'Hello, ' + Name + '!'
end;

begin
  WriteLn(Greet('Alice'))
end.",
    );
    assert_eq!(out.lines, vec!["Hello, Alice!"]);
}

#[test]
fn function_returning_boolean() {
    let out = compile_and_run(
        "\
program FuncBool;
uses Std.Console;

function IsPositive(N: integer): boolean;
begin
  return N > 0
end;

begin
  WriteLn(IsPositive(5));
  WriteLn(IsPositive(-3));
  WriteLn(IsPositive(0))
end.",
    );
    assert_eq!(out.lines, vec!["true", "false", "false"]);
}

#[test]
fn function_returning_real() {
    let out = compile_and_run(
        "\
program FuncReal;
uses Std.Console;

function Half(N: real): real;
begin
  return N / 2.0
end;

begin
  WriteLn(Half(10.0))
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn function_zero_params() {
    let out = compile_and_run(
        "\
program FuncZero;
uses Std.Console;

function FortyTwo(): integer;
begin
  return 42
end;

begin
  WriteLn(FortyTwo())
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

// ═══════════════════════════════════════════════════════════════
// PROCEDURES
// ═══════════════════════════════════════════════════════════════

#[test]
fn procedure_call() {
    let out = compile_and_run(
        "\
program ProcTest;

procedure Greet(name: string);
begin
  Std.Console.WriteLn(name)
end;

begin
  Greet('World')
end.",
    );
    assert_eq!(out.lines, vec!["World"]);
}

#[test]
fn procedure_multiple_params() {
    let out = compile_and_run(
        "\
program ProcMulti;
uses Std.Console;

procedure PrintSum(A: integer; B: integer; C: integer);
begin
  WriteLn(A + B + C)
end;

begin
  PrintSum(1, 2, 3)
end.",
    );
    assert_eq!(out.lines, vec!["6"]);
}

#[test]
fn procedure_zero_params() {
    let out = compile_and_run(
        "\
program ProcZero;
uses Std.Console;

procedure SayHi();
begin
  WriteLn('Hi')
end;

begin
  SayHi()
end.",
    );
    assert_eq!(out.lines, vec!["Hi"]);
}

// ═══════════════════════════════════════════════════════════════
// PARAMETERS
// ═══════════════════════════════════════════════════════════════

#[test]
fn multi_param_function_clamp() {
    let out = compile_and_run(
        "\
program ClampTest;
uses Std.Console;

function Clamp(Value: integer; Min: integer; Max: integer): integer;
begin
  if Value < Min then
    return Min
  else if Value > Max then
    return Max
  else
    return Value
end;

begin
  WriteLn(Clamp(150, 0, 100));
  WriteLn(Clamp(-5, 0, 100));
  WriteLn(Clamp(50, 0, 100))
end.",
    );
    assert_eq!(out.lines, vec!["100", "0", "50"]);
}

// ═══════════════════════════════════════════════════════════════
// RECURSION
// ═══════════════════════════════════════════════════════════════

#[test]
fn recursive_function() {
    let out = compile_and_run(
        "\
program RecurTest;

function Fact(n: integer): integer;
begin
  if n <= 1 then
    return 1
  else
    return n * Fact(n - 1)
end;

begin
  Std.Console.WriteLn(Fact(5))
end.",
    );
    assert_eq!(out.lines, vec!["120"]);
}

#[test]
fn recursive_function_base_case_zero() {
    let out = compile_and_run(
        "\
program RecurBase;
uses Std.Console;

function Fib(N: integer): integer;
begin
  if N <= 1 then
    return N
  else
    return Fib(N - 1) + Fib(N - 2)
end;

begin
  WriteLn(Fib(0));
  WriteLn(Fib(1));
  WriteLn(Fib(10))
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "55"]);
}

// ═══════════════════════════════════════════════════════════════
// EARLY RETURN
// ═══════════════════════════════════════════════════════════════

#[test]
fn early_return_from_loop() {
    let out = compile_and_run(
        "\
program EarlyReturn;
uses Std.Console, Std.Array;

function IndexOf(Items: array of string; Target: string): integer;
begin
  for I: integer := 0 to Std.Array.Length(Items) - 1 do
  begin
    if Items[I] = Target then
      return I
  end;
  return -1
end;

begin
  WriteLn(IndexOf(['a', 'b', 'c'], 'b'));
  WriteLn(IndexOf(['a', 'b', 'c'], 'z'))
end.",
    );
    assert_eq!(out.lines, vec!["1", "-1"]);
}

#[test]
fn early_return_skips_remaining_code() {
    let out = compile_and_run(
        "\
program EarlySkip;
uses Std.Console;

function Check(N: integer): string;
begin
  if N < 0 then
    return 'negative';
  if N = 0 then
    return 'zero';
  return 'positive'
end;

begin
  WriteLn(Check(-5));
  WriteLn(Check(0));
  WriteLn(Check(7))
end.",
    );
    assert_eq!(out.lines, vec!["negative", "zero", "positive"]);
}

// ═══════════════════════════════════════════════════════════════
// FUNCTION TYPE ALIASES
// ═══════════════════════════════════════════════════════════════

#[test]
fn function_type_alias_as_variable() {
    let out = compile_and_run(
        "\
program FuncTypeAlias;
uses Std.Console;

type
  IntBinaryOp = function(A: integer; B: integer): integer;

function Add(A: integer; B: integer): integer;
begin
  return A + B
end;

begin
  var Op: IntBinaryOp := Add;
  WriteLn(Op(3, 4))
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

#[test]
fn function_type_alias_as_parameter() {
    let out = compile_and_run(
        "\
program FuncTypeParam;
uses Std.Console;

type
  IntOp = function(X: integer): integer;

function Apply(F: IntOp; Value: integer): integer;
begin
  return F(Value)
end;

function Triple(X: integer): integer;
begin
  return X * 3
end;

begin
  WriteLn(Apply(Triple, 5))
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}

#[test]
fn procedure_type_alias() {
    let out = compile_and_run(
        "\
program ProcTypeAlias;
uses Std.Console;

type
  StringAction = procedure(S: string);

procedure Shout(S: string);
begin
  WriteLn(S + '!')
end;

begin
  var Action: StringAction := Shout;
  Action('Hello')
end.",
    );
    assert_eq!(out.lines, vec!["Hello!"]);
}

/// Tests for basic function and procedure declarations.
///
/// **Documentation:** [docs/pascal/language/functions/README.md](docs/pascal/language/functions/README.md)
use super::super::*;

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

// ═══════════════════════════════════════════════════════════════
// EDGE CASES
// ═══════════════════════════════════════════════════════════════

#[test]
fn chained_function_calls() {
    let out = compile_and_run(
        "\
program ChainedCalls;
uses Std.Console;

function Double(X: integer): integer;
begin
  return X * 2
end;

function Inc(X: integer): integer;
begin
  return X + 1
end;

begin
  WriteLn(Double(Inc(Double(5))))
end.",
    );
    assert_eq!(out.lines, vec!["22"]);
}

#[test]
fn function_result_in_expression() {
    let out = compile_and_run(
        "\
program FuncInExpr;
uses Std.Console;

function Square(X: integer): integer;
begin
  return X * X
end;

begin
  WriteLn(Square(3) + Square(4))
end.",
    );
    assert_eq!(out.lines, vec!["25"]);
}

#[test]
fn function_many_parameters() {
    let out = compile_and_run(
        "\
program ManyParams;
uses Std.Console;

function Sum5(A: integer; B: integer; C: integer; D: integer; E: integer): integer;
begin
  return A + B + C + D + E
end;

begin
  WriteLn(Sum5(1, 2, 3, 4, 5))
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}

#[test]
fn hypotenuse_from_spec() {
    let out = compile_and_run(
        "\
program HypotenuseTest;
uses Std.Console, Std.Math;

function Hypotenuse(A: real; B: real): real;

  function Square(X: real): real;
  begin
    return X * X
  end;

begin
  return Sqrt(Square(A) + Square(B))
end;

begin
  WriteLn(Hypotenuse(3.0, 4.0))
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn function_calling_other_function() {
    let out = compile_and_run(
        "\
program FuncCallsFunc;
uses Std.Console;

function Add(A: integer; B: integer): integer;
begin
  return A + B
end;

function AddThree(A: integer; B: integer; C: integer): integer;
begin
  return Add(Add(A, B), C)
end;

begin
  WriteLn(AddThree(10, 20, 30))
end.",
    );
    assert_eq!(out.lines, vec!["60"]);
}

#[test]
fn procedure_with_side_effects_only() {
    let out = compile_and_run(
        "\
program ProcSideEffects;
uses Std.Console;

procedure PrintBanner(Title: string);
begin
  WriteLn('=====');
  WriteLn(Title);
  WriteLn('=====')
end;

begin
  PrintBanner('Hello')
end.",
    );
    assert_eq!(out.lines, vec!["=====", "Hello", "====="]);
}

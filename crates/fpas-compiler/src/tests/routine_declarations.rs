/// Tests for routine declaration order and mutual recursion without `forward`.
///
/// **Documentation:** `docs/pascal/04-functions.md` (from the repository root).
use super::*;

#[test]
fn nested_mutual_recursion_is_even_odd() {
    let out = compile_and_run(
        "\
program MutualRec;

function IsEven(n: integer): boolean;
  function IsOdd(x: integer): boolean;
  begin
    if x = 0 then
      return false
    else
      return IsEven(x - 1)
  end;
begin
  if n = 0 then
    return true
  else
    return IsOdd(n - 1)
end;

begin
  Std.Console.WriteLn(IsEven(4));
  Std.Console.WriteLn(IsEven(3));
  Std.Console.WriteLn(IsEven(5));
  Std.Console.WriteLn(IsEven(6))
end.",
    );
    assert_eq!(out.lines, vec!["true", "false", "false", "true"]);
}

#[test]
fn procedure_single_declaration() {
    let out = compile_and_run(
        "\
program ProcDecl;

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
fn callee_declared_before_caller() {
    let out = compile_and_run(
        "\
program CallOrder;

function Helper(x: integer): integer;
begin
  return x * 10
end;

function Main(x: integer): integer;
begin
  return Helper(x) + 1
end;

begin
  Std.Console.WriteLn(Main(5))
end.",
    );
    assert_eq!(out.lines, vec!["51"]);
}

#[test]
fn function_with_multiple_params_no_forward() {
    let out = compile_and_run(
        "\
program MultiParam;
uses Std.Console;

function Combine(A: string; B: string; Sep: string): string;
begin
  return A + Sep + B
end;

begin
  WriteLn(Combine('Hello', 'World', ' '))
end.",
    );
    assert_eq!(out.lines, vec!["Hello World"]);
}

#[test]
fn function_zero_params_no_forward() {
    let out = compile_and_run(
        "\
program ZeroParam;
uses Std.Console;

function GetValue(): integer;
begin
  return 99
end;

begin
  WriteLn(GetValue())
end.",
    );
    assert_eq!(out.lines, vec!["99"]);
}

// ═══════════════════════════════════════════════════════════════
// NEGATIVE — duplicate routines in the same scope
// ═══════════════════════════════════════════════════════════════

#[test]
fn duplicate_function_rejected() {
    let err = compile_err(
        "\
program DupFn;

function Value(): integer;
begin
  return 1
end;

function Value(): integer;
begin
  return 2
end;

begin
  var X: integer := Value()
end.",
    );
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION,
        "unexpected error: {err:?}"
    );
}

#[test]
fn duplicate_function_kind_mismatch_rejected() {
    let err = compile_err(
        "\
program KindMismatch;

procedure LogValue(X: integer);
begin
  return
end;

function LogValue(X: integer): integer;
begin
  return X
end;

begin
  var X: integer := LogValue(1)
end.",
    );
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION,
        "unexpected error: {err:?}"
    );
}

#[test]
fn duplicate_procedure_vs_function_rejected() {
    let err = compile_err(
        "\
program KindMismatch2;

function GetValue(): integer;
begin
  return 1
end;

procedure GetValue();
begin
  Std.Console.WriteLn('wrong')
end;

begin
  GetValue()
end.",
    );
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION,
        "unexpected error: {err:?}"
    );
}

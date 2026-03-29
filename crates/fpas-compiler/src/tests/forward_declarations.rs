use super::*;

// ═══════════════════════════════════════════════════════════════
// POSITIVE — forward declarations
// ═══════════════════════════════════════════════════════════════

#[test]
fn forward_mutual_recursion_is_even_odd() {
    let out = compile_and_run(
        "\
program MutualRec;

function IsEven(n: integer): boolean; forward;
function IsOdd(n: integer): boolean; forward;

function IsEven(n: integer): boolean;
begin
  if n = 0 then
    return true
  else
    return IsOdd(n - 1)
end;

function IsOdd(n: integer): boolean;
begin
  if n = 0 then
    return false
  else
    return IsEven(n - 1)
end;

begin
  Std.Console.WriteLn(IsEven(4));
  Std.Console.WriteLn(IsEven(3));
  Std.Console.WriteLn(IsOdd(5));
  Std.Console.WriteLn(IsOdd(6))
end.",
    );
    assert_eq!(out.lines, vec!["true", "false", "true", "false"]);
}

#[test]
fn forward_procedure() {
    let out = compile_and_run(
        "\
program ForwardProc;

procedure Greet(name: string); forward;

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
fn forward_call_before_implementation() {
    let out = compile_and_run(
        "\
program ForwardOrder;

function Helper(x: integer): integer; forward;

function Main(x: integer): integer;
begin
  return Helper(x) + 1
end;

function Helper(x: integer): integer;
begin
  return x * 10
end;

begin
  Std.Console.WriteLn(Main(5))
end.",
    );
    assert_eq!(out.lines, vec!["51"]);
}

#[test]
fn forward_with_multiple_params() {
    let out = compile_and_run(
        "\
program ForwardMulti;
uses Std.Console;

function Combine(A: string; B: string; Sep: string): string; forward;

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
fn forward_zero_params() {
    let out = compile_and_run(
        "\
program ForwardZero;
uses Std.Console;

function GetValue(): integer; forward;

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
// NEGATIVE — forward declarations
// ═══════════════════════════════════════════════════════════════

#[test]
fn forward_function_requires_implementation() {
    let err = compile_err(
        "\
program MissingForwardImpl;

function Value(): integer; forward;

begin
  var X: integer := Value()
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_UNKNOWN_NAME);
    assert!(err.message.contains("forward"), "unexpected error: {err:?}");
}

#[test]
fn forward_function_signature_must_match_implementation() {
    let err = compile_err(
        "\
program ForwardMismatch;

function Value(X: integer): integer; forward;

function Value(X: integer): string;
begin
  return 'wrong'
end;

begin
  var X: string := Value(1)
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Forward declaration"),
        "unexpected error: {err:?}"
    );
}

#[test]
fn forward_procedure_cannot_become_function() {
    let err = compile_err(
        "\
program ForwardKindMismatch;

procedure LogValue(X: integer); forward;

function LogValue(X: integer): integer;
begin
  return X
end;

begin
  var X: integer := LogValue(1)
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Forward declaration"),
        "unexpected error: {err:?}"
    );
}

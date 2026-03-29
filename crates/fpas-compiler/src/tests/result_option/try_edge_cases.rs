/// Edge cases and negative tests for the `try` operator — `docs/pascal/07-error-handling.md`.
use super::{compile_and_run, compile_err};

// ── Error content preservation through try ──────────────────────────────

#[test]
fn try_preserves_error_content() {
    let out = compile_and_run(
        "program T;
function Inner(): Result of integer, string;
begin
  return Error('specific error')
end;
function Outer(): Result of string, string;
begin
  var V: integer := try Inner();
  return Ok(Std.Conv.IntToStr(V))
end;
begin
  case Outer() of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["specific error"]);
}

// ── Multiple try in one function ────────────────────────────────────────

#[test]
fn try_chained_multiple_calls() {
    let out = compile_and_run(
        "program T;
function GetA(): Result of integer, string;
begin
  return Ok(10)
end;
function GetB(): Result of integer, string;
begin
  return Ok(20)
end;
function Combined(): Result of integer, string;
begin
  var A: integer := try GetA();
  var B: integer := try GetB();
  return Ok(A + B)
end;
begin
  Std.Console.WriteLn(Std.Result.Unwrap(Combined()))
end.",
    );
    assert_eq!(out.lines, vec!["30"]);
}

#[test]
fn try_chained_first_fails() {
    let out = compile_and_run(
        "program T;
function GetA(): Result of integer, string;
begin
  return Error('A failed')
end;
function GetB(): Result of integer, string;
begin
  return Ok(20)
end;
function Combined(): Result of integer, string;
begin
  var A: integer := try GetA();
  var B: integer := try GetB();
  return Ok(A + B)
end;
begin
  case Combined() of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["A failed"]);
}

#[test]
fn try_chained_second_fails() {
    let out = compile_and_run(
        "program T;
function GetA(): Result of integer, string;
begin
  return Ok(10)
end;
function GetB(): Result of integer, string;
begin
  return Error('B failed')
end;
function Combined(): Result of integer, string;
begin
  var A: integer := try GetA();
  var B: integer := try GetB();
  return Ok(A + B)
end;
begin
  case Combined() of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["B failed"]);
}

// ── Try in expression context ───────────────────────────────────────────

#[test]
fn try_result_used_in_arithmetic() {
    let out = compile_and_run(
        "program T;
function GetVal(): Result of integer, string;
begin
  return Ok(5)
end;
function Compute(): Result of integer, string;
begin
  return Ok(try GetVal() * 3)
end;
begin
  Std.Console.WriteLn(Std.Result.Unwrap(Compute()))
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}

// ── Process example from docs ───────────────────────────────────────────

#[test]
fn try_process_example_ok() {
    let out = compile_and_run(
        "program T;
function Divide(A: integer; B: integer): Result of integer, string;
begin
  if B = 0 then
    return Error('Division by zero')
  else
    return Ok(A div B)
end;
function Process(A: integer; B: integer): Result of string, string;
begin
  var Quotient: integer := try Divide(A, B);
  return Ok(Std.Conv.IntToStr(Quotient))
end;
begin
  case Process(10, 2) of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn try_process_example_error_propagation() {
    let out = compile_and_run(
        "program T;
function Divide(A: integer; B: integer): Result of integer, string;
begin
  if B = 0 then
    return Error('Division by zero')
  else
    return Ok(A div B)
end;
function Process(A: integer; B: integer): Result of string, string;
begin
  var Quotient: integer := try Divide(A, B);
  return Ok(Std.Conv.IntToStr(Quotient))
end;
begin
  case Process(10, 0) of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["Division by zero"]);
}

// ── Try with Option — FirstPositive example from docs ───────────────────

#[test]
fn try_option_first_positive_found() {
    let out = compile_and_run(
        "program T;
function FindIndex(Items: array of integer; Target: integer): Option of integer;
begin
  for I: integer := 0 to Std.Array.Length(Items) - 1 do
    if Items[I] = Target then
      return Some(I);
  return None
end;
function FirstPositive(Items: array of integer): Option of integer;
begin
  var Idx: integer := try FindIndex(Items, 1);
  return Some(Items[Idx])
end;
begin
  case FirstPositive([0, 1, 2]) of
    Some(V): Std.Console.WriteLn(V);
    None:    Std.Console.WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["1"]);
}

#[test]
fn try_option_first_positive_not_found() {
    let out = compile_and_run(
        "program T;
function FindIndex(Items: array of integer; Target: integer): Option of integer;
begin
  for I: integer := 0 to Std.Array.Length(Items) - 1 do
    if Items[I] = Target then
      return Some(I);
  return None
end;
function FirstPositive(Items: array of integer): Option of integer;
begin
  var Idx: integer := try FindIndex(Items, 1);
  return Some(Items[Idx])
end;
begin
  case FirstPositive([0, 2, 3]) of
    Some(V): Std.Console.WriteLn(V);
    None:    Std.Console.WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["none"]);
}

// ── Negative: try on non-Result/Option ──────────────────────────────────

#[test]
fn try_on_integer_is_compile_error() {
    let err = compile_err(
        "program T;
function Foo(): Result of integer, string;
begin
  var X: integer := try 42;
  return Ok(X)
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

#[test]
fn try_on_string_is_compile_error() {
    let err = compile_err(
        "program T;
function Foo(): Result of string, string;
begin
  var X: string := try 'hello';
  return Ok(X)
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

#[test]
fn try_on_boolean_is_compile_error() {
    let err = compile_err(
        "program T;
function Foo(): Result of boolean, string;
begin
  var X: boolean := try true;
  return Ok(X)
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

#[test]
fn try_result_in_plain_function_is_compile_error() {
    let err = compile_err(
        "program T;
function Foo(): integer;
begin
  var X: integer := try Ok(42);
  return X
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("returns `Integer`") || err.message.contains("returns `integer`"),
        "expected enclosing return type mismatch, got: {err:?}"
    );
}

#[test]
fn try_option_in_plain_function_is_compile_error() {
    let err = compile_err(
        "program T;
function Foo(): integer;
begin
  var X: integer := try Some(42);
  return X
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("returns `Integer`") || err.message.contains("returns `integer`"),
        "expected enclosing return type mismatch, got: {err:?}"
    );
}

#[test]
fn try_result_in_option_function_is_compile_error() {
    let err = compile_err(
        "program T;
function Foo(): Option of integer;
begin
  var X: integer := try Ok(42);
  return Some(X)
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Option") && err.message.contains("Result"),
        "expected Result/Option mismatch, got: {err:?}"
    );
}

#[test]
fn try_option_in_result_function_is_compile_error() {
    let err = compile_err(
        "program T;
function Foo(): Result of integer, string;
begin
  var X: integer := try Some(42);
  return Ok(X)
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Option") && err.message.contains("Result"),
        "expected Option/Result mismatch, got: {err:?}"
    );
}

#[test]
fn try_result_error_type_mismatch_is_compile_error() {
    let err = compile_err(
        "program T;
function Inner(): Result of integer, string;
begin
  return Error('boom')
end;
function Outer(): Result of integer, integer;
begin
  var X: integer := try Inner();
  return Ok(X)
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Result") && err.message.contains("Outer"),
        "expected Result error-type mismatch, got: {err:?}"
    );
}

#[test]
fn try_in_procedure_is_compile_error() {
    let err = compile_err(
        "program T;
procedure Foo();
begin
  var X: integer := try Some(42)
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Procedure `Foo`"),
        "expected procedure-specific diagnostic, got: {err:?}"
    );
}

// ── Try in program main block ───────────────────────────────────────────

#[test]
fn try_result_in_program_main_block_is_compile_error() {
    let err = compile_err(
        "program T;
function GetVal(): Result of integer, string;
begin
  return Ok(42)
end;
begin
  var X: integer := try GetVal()
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

#[test]
fn try_option_in_program_main_block_is_compile_error() {
    let err = compile_err(
        "program T;
function MaybeVal(): Option of integer;
begin
  return Some(7)
end;
begin
  var X: integer := try MaybeVal()
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

// ── Nested try expressions ──────────────────────────────────────────────

#[test]
fn try_nested_in_function_call() {
    let out = compile_and_run(
        "program T;
function GetDivisor(): Result of integer, string;
begin
  return Ok(2)
end;
function GetDividend(): Result of integer, string;
begin
  return Ok(10)
end;
function Compute(): Result of integer, string;
begin
  return Ok(try GetDividend() div try GetDivisor())
end;
begin
  Std.Console.WriteLn(Std.Result.Unwrap(Compute()))
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn try_nested_first_fails() {
    let out = compile_and_run(
        "program T;
function GetDivisor(): Result of integer, string;
begin
  return Ok(2)
end;
function GetDividend(): Result of integer, string;
begin
  return Error('no dividend')
end;
function Compute(): Result of integer, string;
begin
  return Ok(try GetDividend() div try GetDivisor())
end;
begin
  case Compute() of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["no dividend"]);
}

#[test]
fn try_nested_second_fails() {
    let out = compile_and_run(
        "program T;
function GetDivisor(): Result of integer, string;
begin
  return Error('no divisor')
end;
function GetDividend(): Result of integer, string;
begin
  return Ok(10)
end;
function Compute(): Result of integer, string;
begin
  return Ok(try GetDividend() div try GetDivisor())
end;
begin
  case Compute() of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["no divisor"]);
}

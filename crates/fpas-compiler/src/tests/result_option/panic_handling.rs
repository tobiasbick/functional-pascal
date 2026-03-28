/// Tests for the `panic` statement from `docs/pascal/07-error-handling.md`.
use super::{compile_and_run, compile_err, compile_run_err, compile_run_error};

// ── Happy path ──────────────────────────────────────────────────────────

#[test]
fn panic_with_string_literal() {
    let msg = compile_run_err(
        "program T;
begin
  panic('Something went terribly wrong')
end.",
    );
    assert!(
        msg.contains("Something went terribly wrong"),
        "Expected panic message, got: {msg}",
    );
}

#[test]
fn panic_has_runtime_program_panic_code() {
    let err = compile_run_error(
        "program T;
begin
  panic('boom')
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
}

#[test]
fn panic_halts_execution() {
    let msg = compile_run_err(
        "program T;
begin
  Std.Console.WriteLn('before');
  panic('halt');
  Std.Console.WriteLn('after')
end.",
    );
    assert!(msg.contains("halt"), "Expected panic message, got: {msg}");
    // 'after' should never print – if it did the VM would have succeeded,
    // but we got a runtime error so the panic halted execution.
}

#[test]
fn panic_with_string_expression() {
    let msg = compile_run_err(
        "program T;
var Code: integer := 42;
begin
  panic('Error code: ' + Std.Conv.IntToStr(Code))
end.",
    );
    assert!(
        msg.contains("Error code: 42"),
        "Expected concatenated message, got: {msg}",
    );
}

// ── Guard pattern from docs ─────────────────────────────────────────────

#[test]
fn panic_in_guard_function() {
    let out = compile_and_run(
        "program T;
function DivideChecked(A: integer; B: integer): integer;
begin
  if B = 0 then
    panic('Division by zero');
  return A div B
end;
begin
  Std.Console.WriteLn(DivideChecked(10, 2))
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn panic_in_guard_function_triggers_on_zero() {
    let err = compile_run_error(
        "program T;
function DivideChecked(A: integer; B: integer): integer;
begin
  if B = 0 then
    panic('Division by zero');
  return A div B
end;
begin
  Std.Console.WriteLn(DivideChecked(10, 0))
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
    assert!(
        err.message.contains("Division by zero"),
        "Expected message, got: {}",
        err.message,
    );
}

// ── Negative / compile-time errors ──────────────────────────────────────

#[test]
fn panic_with_integer_is_compile_error() {
    let err = compile_err(
        "program T;
begin
  panic(42)
end.",
    );
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::SEMA_INVALID_PANIC_ARGUMENT
    );
}

#[test]
fn panic_with_boolean_is_compile_error() {
    let err = compile_err(
        "program T;
begin
  panic(true)
end.",
    );
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::SEMA_INVALID_PANIC_ARGUMENT
    );
}

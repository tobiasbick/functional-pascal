/// Tests for the `panic` statement from `docs/pascal/07-error-handling.md`.
use super::{compile_and_run, compile_err, compile_run_err, compile_run_error, parse_fails};

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

#[test]
fn panic_with_empty_string() {
    let err = compile_run_error(
        "program T;
begin
  panic('')
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
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

// ── Case-else pattern from docs ─────────────────────────────────────────

#[test]
fn panic_in_case_else_branch_happy_path() {
    let out = compile_and_run(
        "program T;
function DayKind(Day: string): string;
begin
  case Day of
    'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday':
      return 'Weekday';
    'Saturday', 'Sunday':
      return 'Weekend'
  else
    panic('Invalid day: ' + Day)
  end
end;
begin
  Std.Console.WriteLn(DayKind('Monday'));
  Std.Console.WriteLn(DayKind('Saturday'))
end.",
    );
    assert_eq!(out.lines, vec!["Weekday", "Weekend"]);
}

#[test]
fn panic_in_case_else_branch_triggers() {
    let err = compile_run_error(
        "program T;
function DayKind(Day: string): string;
begin
  case Day of
    'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday':
      return 'Weekday';
    'Saturday', 'Sunday':
      return 'Weekend'
  else
    panic('Invalid day: ' + Day)
  end
end;
begin
  Std.Console.WriteLn(DayKind('Funday'))
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
    assert!(
        err.message.contains("Invalid day: Funday"),
        "Expected message, got: {}",
        err.message,
    );
}

// ── Panic in nested function ────────────────────────────────────────────

#[test]
fn panic_in_nested_function() {
    let err = compile_run_error(
        "program T;
function Outer(): integer;
  function Inner(X: integer): integer;
  begin
    if X < 0 then panic('negative');
    return X
  end;
begin
  return Inner(-1)
end;
begin
  Std.Console.WriteLn(Outer())
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
    assert!(
        err.message.contains("negative"),
        "Expected message, got: {}",
        err.message,
    );
}

// ── Panic in loop body ──────────────────────────────────────────────────

#[test]
fn panic_in_while_loop_body() {
    let err = compile_run_error(
        "program T;
mutable var I: integer := 0;
begin
  while I < 5 do
  begin
    if I = 3 then panic('hit 3');
    I := I + 1
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
    assert!(
        err.message.contains("hit 3"),
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

#[test]
fn panic_with_real_is_compile_error() {
    let err = compile_err(
        "program T;
begin
  panic(3.14)
end.",
    );
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::SEMA_INVALID_PANIC_ARGUMENT
    );
}

#[test]
fn panic_no_argument_is_parse_error() {
    parse_fails(
        "program T;
begin
  panic
end.",
    );
}

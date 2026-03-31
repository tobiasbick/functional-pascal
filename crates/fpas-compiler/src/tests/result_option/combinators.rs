/// Tests for `Std.Result.{Map,AndThen,OrElse}` and `Std.Option.{Map,AndThen,OrElse}` —
/// `docs/pascal/std/result.md` and `docs/pascal/std/option.md`.
use super::{compile_and_run, compile_err};

// ── Result.Map ──────────────────────────────────────────────────────────

#[test]
fn result_map_ok_transforms_value() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function DoubleToStr(V: integer): string;
begin
  return IntToStr(V * 2)
end;
begin
  var R: Result of integer, string := Ok(21);
  var M: Result of string, string := Map(R, DoubleToStr);
  case M of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn result_map_error_passes_through() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function ToStr(V: integer): string;
begin
  return IntToStr(V)
end;
begin
  var R: Result of integer, string := Error('fail');
  var M: Result of string, string := Map(R, ToStr);
  case M of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["fail"]);
}

// ── Result.AndThen ──────────────────────────────────────────────────────

#[test]
fn result_and_then_ok_chains() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function PositiveToStr(V: integer): Result of string, string;
begin
  if V > 0 then return Ok(IntToStr(V))
  else return Error('non-positive')
end;
begin
  var R: Result of integer, string := Ok(10);
  var M: Result of string, string := AndThen(R, PositiveToStr);
  case M of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

#[test]
fn result_and_then_ok_produces_error() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function PositiveToStr(V: integer): Result of string, string;
begin
  if V > 0 then return Ok(IntToStr(V))
  else return Error('non-positive')
end;
begin
  var R: Result of integer, string := Ok(-5);
  var M: Result of string, string := AndThen(R, PositiveToStr);
  case M of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["non-positive"]);
}

#[test]
fn result_and_then_error_passes_through() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function WrapOk(V: integer): Result of string, string;
begin
  return Ok(IntToStr(V))
end;
begin
  var R: Result of integer, string := Error('early');
  var M: Result of string, string := AndThen(R, WrapOk);
  case M of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["early"]);
}

// ── Result.OrElse ───────────────────────────────────────────────────────

#[test]
fn result_or_else_ok_passes_through() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function RecoverZero(E: string): Result of integer, string;
begin
  return Ok(0)
end;
begin
  var R: Result of integer, string := Ok(42);
  var M: Result of integer, string := OrElse(R, RecoverZero);
  case M of
    Ok(V): WriteLn(IntToStr(V));
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn result_or_else_error_recovers() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function RecoverNinetyNine(E: string): Result of integer, string;
begin
  return Ok(99)
end;
begin
  var R: Result of integer, string := Error('oops');
  var M: Result of integer, string := OrElse(R, RecoverNinetyNine);
  case M of
    Ok(V): WriteLn(IntToStr(V));
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["99"]);
}

// ── Option.Map ──────────────────────────────────────────────────────────

#[test]
fn option_map_some_transforms_value() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function TripleToStr(V: integer): string;
begin
  return IntToStr(V * 3)
end;
begin
  var O: Option of integer := Some(7);
  var M: Option of string := Map(O, TripleToStr);
  case M of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["21"]);
}

#[test]
fn option_map_none_passes_through() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function ToStr(V: integer): string;
begin
  return IntToStr(V)
end;
begin
  var O: Option of integer := None;
  var M: Option of string := Map(O, ToStr);
  case M of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["none"]);
}

// ── Option.AndThen ──────────────────────────────────────────────────────

#[test]
fn option_and_then_some_chains() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function PositiveToStr(V: integer): Option of string;
begin
  if V > 0 then return Some(IntToStr(V))
  else return None
end;
begin
  var O: Option of integer := Some(5);
  var M: Option of string := AndThen(O, PositiveToStr);
  case M of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn option_and_then_some_returns_none() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function PositiveToStr(V: integer): Option of string;
begin
  if V > 0 then return Some(IntToStr(V))
  else return None
end;
begin
  var O: Option of integer := Some(-1);
  var M: Option of string := AndThen(O, PositiveToStr);
  case M of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["none"]);
}

#[test]
fn option_and_then_none_passes_through() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function ToSomeStr(V: integer): Option of string;
begin
  return Some(IntToStr(V))
end;
begin
  var O: Option of integer := None;
  var M: Option of string := AndThen(O, ToSomeStr);
  case M of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["none"]);
}

// ── Option.OrElse ───────────────────────────────────────────────────────

#[test]
fn option_or_else_some_passes_through() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function FallbackZero(): Option of integer;
begin
  return Some(0)
end;
begin
  var O: Option of integer := Some(42);
  var M: Option of integer := OrElse(O, FallbackZero);
  case M of
    Some(V): WriteLn(IntToStr(V));
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn option_or_else_none_provides_fallback() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function FallbackNinetyNine(): Option of integer;
begin
  return Some(99)
end;
begin
  var O: Option of integer := None;
  var M: Option of integer := OrElse(O, FallbackNinetyNine);
  case M of
    Some(V): WriteLn(IntToStr(V));
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["99"]);
}

// ── Chaining ────────────────────────────────────────────────────────────

#[test]
fn result_map_and_then_chain() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function MulTen(V: integer): integer;
begin
  return V * 10
end;
function WrapInOk(V: integer): Result of string, string;
begin
  return Ok(IntToStr(V))
end;
begin
  var R: Result of integer, string := Ok(5);
  var Step1: Result of integer, string := Map(R, MulTen);
  var Step2: Result of string, string := AndThen(Step1, WrapInOk);
  case Step2 of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["50"]);
}

#[test]
fn option_map_and_then_chain() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function AddSeven(V: integer): integer;
begin
  return V + 7
end;
function WrapInSome(V: integer): Option of string;
begin
  return Some(IntToStr(V))
end;
begin
  var O: Option of integer := Some(3);
  var Step1: Option of integer := Map(O, AddSeven);
  var Step2: Option of string := AndThen(Step1, WrapInSome);
  case Step2 of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

// ── Qualified calls ─────────────────────────────────────────────────────

#[test]
fn result_map_qualified_call() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function MulSeven(V: integer): integer;
begin
  return V * 7
end;
begin
  var R: Result of integer, string := Ok(6);
  var M: Result of integer, string := Std.Result.Map(R, MulSeven);
  case M of
    Ok(V): WriteLn(IntToStr(V));
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn option_and_then_qualified_call() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function ToSomeStr(V: integer): Option of string;
begin
  return Some(IntToStr(V))
end;
begin
  var O: Option of integer := Some(10);
  var M: Option of string := Std.Option.AndThen(O, ToSomeStr);
  case M of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

// ── Closures capture enclosing variables ────────────────────────────────

#[test]
fn result_map_closure_captures_variable() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function MultiplyByHundred(V: integer): integer;
begin
  return V * 100
end;
begin
  var R: Result of integer, string := Ok(3);
  var M: Result of integer, string := Map(R, MultiplyByHundred);
  case M of
    Ok(V): WriteLn(IntToStr(V));
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["300"]);
}

#[test]
fn option_map_closure_captures_variable() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function PrefixedToStr(V: integer): string;
begin
  return 'val=' + IntToStr(V)
end;
begin
  var O: Option of integer := Some(42);
  var M: Option of string := Map(O, PrefixedToStr);
  case M of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["val=42"]);
}

// ── Deeper chaining (3+ steps, error short-circuits) ────────────────────

#[test]
fn result_three_step_chain_ok() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function AddThree(V: integer): integer;
begin
  return V + 3
end;
function MulTen(V: integer): integer;
begin
  return V * 10
end;
function WrapResult(V: integer): Result of string, string;
begin
  return Ok('result=' + IntToStr(V))
end;
begin
  var R: Result of integer, string := Ok(2);
  var S1: Result of integer, string := Map(R, AddThree);
  var S2: Result of integer, string := Map(S1, MulTen);
  var S3: Result of string, string := AndThen(S2, WrapResult);
  case S3 of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["result=50"]);
}

#[test]
fn result_three_step_chain_error_short_circuits() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function AddThree(V: integer): integer;
begin
  return V + 3
end;
function MulTen(V: integer): integer;
begin
  return V * 10
end;
function WrapResult(V: integer): Result of string, string;
begin
  return Ok(IntToStr(V))
end;
begin
  var R: Result of integer, string := Error('boom');
  var S1: Result of integer, string := Map(R, AddThree);
  var S2: Result of integer, string := Map(S1, MulTen);
  var S3: Result of string, string := AndThen(S2, WrapResult);
  case S3 of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["boom"]);
}

#[test]
fn option_three_step_chain_none_short_circuits() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function AddOne(V: integer): integer;
begin
  return V + 1
end;
function MulTwo(V: integer): integer;
begin
  return V * 2
end;
function WrapSome(V: integer): Option of string;
begin
  return Some(IntToStr(V))
end;
begin
  var O: Option of integer := None;
  var S1: Option of integer := Map(O, AddOne);
  var S2: Option of integer := Map(S1, MulTwo);
  var S3: Option of string := AndThen(S2, WrapSome);
  case S3 of
    Some(S): WriteLn(S);
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["none"]);
}

// ── OrElse callback returns Error/None ──────────────────────────────────

#[test]
fn result_or_else_error_to_error() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result;
function ReplaceError(E: string): Result of integer, string;
begin
  return Error('replaced: ' + E)
end;
begin
  var R: Result of integer, string := Error('first');
  var M: Result of integer, string := OrElse(R, ReplaceError);
  case M of
    Ok(V): WriteLn('ok');
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["replaced: first"]);
}

#[test]
fn option_or_else_none_returns_none() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option;
function AlwaysNone(): Option of integer;
begin
  return None
end;
begin
  var O: Option of integer := None;
  var M: Option of integer := OrElse(O, AlwaysNone);
  case M of
    Some(V): WriteLn('some');
    None: WriteLn('still-none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["still-none"]);
}

// ── Identity map (callback returns input unchanged) ─────────────────────

#[test]
fn result_map_identity() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function Identity(V: integer): integer;
begin
  return V
end;
begin
  var R: Result of integer, string := Ok(42);
  var M: Result of integer, string := Map(R, Identity);
  case M of
    Ok(V): WriteLn(IntToStr(V));
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn option_map_identity() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function Identity(V: integer): integer;
begin
  return V
end;
begin
  var O: Option of integer := Some(7);
  var M: Option of integer := Map(O, Identity);
  case M of
    Some(V): WriteLn(IntToStr(V));
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

// ── Nested Result/Option ────────────────────────────────────────────────

#[test]
fn result_map_nested_option_inner() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Option, Std.Conv;
function WrapInSome(V: integer): Option of integer;
begin
  return Some(V * 2)
end;
begin
  var R: Result of integer, string := Ok(5);
  var M: Result of Option of integer, string := Std.Result.Map(R, WrapInSome);
  case M of
    Ok(Inner):
      case Inner of
        Some(V): WriteLn(IntToStr(V));
        None: WriteLn('inner-none')
      end;
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

#[test]
fn option_map_wraps_in_result() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Option, Std.Conv;
function WrapInOk(V: integer): Result of integer, string;
begin
  return Ok(V * 4)
end;
begin
  var O: Option of integer := Some(3);
  var M: Option of Result of integer, string := Std.Option.Map(O, WrapInOk);
  case M of
    Some(Inner):
      case Inner of
        Ok(V): WriteLn(IntToStr(V));
        Error(E): WriteLn(E)
      end;
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["12"]);
}

// ── Combinator with named function ──────────────────────────────────────

#[test]
fn result_map_with_named_function() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;

function DoubleIt(V: integer): integer;
begin
  return V * 2
end;

begin
  var R: Result of integer, string := Ok(21);
  var M: Result of integer, string := Map(R, DoubleIt);
  case M of
    Ok(V): WriteLn(IntToStr(V));
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn option_and_then_with_named_function() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;

function SafeHalf(V: integer): Option of integer;
begin
  if V mod 2 = 0 then return Some(V div 2)
  else return None
end;

begin
  var O: Option of integer := Some(10);
  var M: Option of integer := AndThen(O, SafeHalf);
  case M of
    Some(V): WriteLn(IntToStr(V));
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn option_and_then_named_function_returns_none() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;

function SafeHalf(V: integer): Option of integer;
begin
  if V mod 2 = 0 then return Some(V div 2)
  else return None
end;

begin
  var O: Option of integer := Some(7);
  var M: Option of integer := AndThen(O, SafeHalf);
  case M of
    Some(V): WriteLn(IntToStr(V));
    None: WriteLn('odd')
  end
end.",
    );
    assert_eq!(out.lines, vec!["odd"]);
}

// ── Combined Result + Option combinators ────────────────────────────────

#[test]
fn result_and_option_combinators_interleaved() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Option, Std.Conv;

function ParsePositive(S: string): Result of integer, string;
begin
  var N: integer := StrToInt(S);
  if N > 0 then return Ok(N)
  else return Error('non-positive')
end;

function ToSomeIfBig(V: integer): Option of integer;
begin
  if V > 10 then return Some(V)
  else return None
end;

begin
  var R: Result of integer, string := ParsePositive('42');
  var Mapped: Result of Option of integer, string := Std.Result.Map(R, ToSomeIfBig);
  case Mapped of
    Ok(Inner):
      case Inner of
        Some(V): WriteLn('big: ' + IntToStr(V));
        None: WriteLn('small')
      end;
    Error(E): WriteLn('error: ' + E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["big: 42"]);
}

// ── Map with type transformation (different input/output types) ─────────

#[test]
fn result_map_integer_to_boolean() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result;
function IsPositive(V: integer): boolean;
begin
  return V > 0
end;
begin
  var R: Result of integer, string := Ok(0);
  var M: Result of boolean, string := Map(R, IsPositive);
  case M of
    Ok(B): if B then WriteLn('positive') else WriteLn('non-positive');
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["non-positive"]);
}

#[test]
fn option_map_string_to_integer() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function ParseAndIncr(S: string): integer;
begin
  return StrToInt(S) + 1
end;
begin
  var O: Option of string := Some('123');
  var M: Option of integer := Map(O, ParseAndIncr);
  case M of
    Some(V): WriteLn(IntToStr(V));
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["124"]);
}

// ── Compile-time error: wrong value type ─────────────────────────────

#[test]
fn result_map_on_non_result_panics() {
    let msg = &compile_err(
        "program T;
uses Std.Result;
function ToSelf(V: integer): integer;
begin
  return V
end;
begin
  var X: integer := 42;
  Std.Result.Map(X, ToSelf)
end.",
    )
    .message;
    assert!(
        msg.contains("must be a Result"),
        "expected type error, got: {msg}"
    );
}

#[test]
fn option_map_on_non_option_panics() {
    let msg = &compile_err(
        "program T;
uses Std.Option;
function ToSelf(V: string): string;
begin
  return V
end;
begin
  var X: string := 'hello';
  Std.Option.Map(X, ToSelf)
end.",
    )
    .message;
    assert!(
        msg.contains("must be an Option"),
        "expected type error, got: {msg}"
    );
}

#[test]
fn result_and_then_on_non_result_panics() {
    let msg = &compile_err(
        "program T;
uses Std.Result;
function ToOk(V: boolean): Result of boolean, string;
begin
  return Ok(V)
end;
begin
  var X: boolean := true;
  Std.Result.AndThen(X, ToOk)
end.",
    )
    .message;
    assert!(
        msg.contains("must be a Result"),
        "expected type error, got: {msg}"
    );
}

#[test]
fn option_and_then_on_non_option_panics() {
    let msg = &compile_err(
        "program T;
uses Std.Option;
function ToSome(V: integer): Option of integer;
begin
  return Some(V)
end;
begin
  var X: integer := 1;
  Std.Option.AndThen(X, ToSome)
end.",
    )
    .message;
    assert!(
        msg.contains("must be an Option"),
        "expected type error, got: {msg}"
    );
}

#[test]
fn result_or_else_on_non_result_panics() {
    let msg = &compile_err(
        "program T;
uses Std.Result;
function RecoverOk(E: integer): Result of integer, string;
begin
  return Ok(0)
end;
begin
  var X: integer := 0;
  Std.Result.OrElse(X, RecoverOk)
end.",
    )
    .message;
    assert!(
        msg.contains("must be a Result"),
        "expected type error, got: {msg}"
    );
}

#[test]
fn option_or_else_on_non_option_panics() {
    let msg = &compile_err(
        "program T;
uses Std.Option;
function FallbackZero(): Option of integer;
begin
  return Some(0)
end;
begin
  var X: integer := 0;
  Std.Option.OrElse(X, FallbackZero)
end.",
    )
    .message;
    assert!(
        msg.contains("must be an Option"),
        "expected type error, got: {msg}"
    );
}

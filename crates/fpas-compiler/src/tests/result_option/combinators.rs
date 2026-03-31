/// Tests for `Std.Result.{Map,AndThen,OrElse}` and `Std.Option.{Map,AndThen,OrElse}` —
/// `docs/pascal/std/result.md` and `docs/pascal/std/option.md`.
use super::{compile_and_run, compile_err};

// ── Result.Map ──────────────────────────────────────────────────────────

#[test]
fn result_map_ok_transforms_value() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
begin
  var R: Result of integer, string := Ok(21);
  var M: Result of string, string := Map(R, function(V: integer): string begin return IntToStr(V * 2) end);
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
begin
  var R: Result of integer, string := Error('fail');
  var M: Result of string, string := Map(R, function(V: integer): string begin return IntToStr(V) end);
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
begin
  var R: Result of integer, string := Ok(10);
  var M: Result of string, string := AndThen(R,
    function(V: integer): Result of string, string
    begin
      if V > 0 then return Ok(IntToStr(V))
      else return Error('non-positive')
    end);
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
begin
  var R: Result of integer, string := Ok(-5);
  var M: Result of string, string := AndThen(R,
    function(V: integer): Result of string, string
    begin
      if V > 0 then return Ok(IntToStr(V))
      else return Error('non-positive')
    end);
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
begin
  var R: Result of integer, string := Error('early');
  var M: Result of string, string := AndThen(R,
    function(V: integer): Result of string, string begin return Ok(IntToStr(V)) end);
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
begin
  var R: Result of integer, string := Ok(42);
  var M: Result of integer, string := OrElse(R,
    function(E: string): Result of integer, string begin return Ok(0) end);
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
begin
  var R: Result of integer, string := Error('oops');
  var M: Result of integer, string := OrElse(R,
    function(E: string): Result of integer, string begin return Ok(99) end);
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
begin
  var O: Option of integer := Some(7);
  var M: Option of string := Map(O, function(V: integer): string begin return IntToStr(V * 3) end);
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
begin
  var O: Option of integer := None;
  var M: Option of string := Map(O, function(V: integer): string begin return IntToStr(V) end);
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
begin
  var O: Option of integer := Some(5);
  var M: Option of string := AndThen(O,
    function(V: integer): Option of string
    begin
      if V > 0 then return Some(IntToStr(V))
      else return None
    end);
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
begin
  var O: Option of integer := Some(-1);
  var M: Option of string := AndThen(O,
    function(V: integer): Option of string
    begin
      if V > 0 then return Some(IntToStr(V))
      else return None
    end);
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
begin
  var O: Option of integer := None;
  var M: Option of string := AndThen(O,
    function(V: integer): Option of string begin return Some(IntToStr(V)) end);
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
begin
  var O: Option of integer := Some(42);
  var M: Option of integer := OrElse(O,
    function(): Option of integer begin return Some(0) end);
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
begin
  var O: Option of integer := None;
  var M: Option of integer := OrElse(O,
    function(): Option of integer begin return Some(99) end);
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
begin
  var R: Result of integer, string := Ok(5);
  var Step1: Result of integer, string := Map(R, function(V: integer): integer begin return V * 10 end);
  var Step2: Result of string, string := AndThen(Step1,
    function(V: integer): Result of string, string begin return Ok(IntToStr(V)) end);
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
begin
  var O: Option of integer := Some(3);
  var Step1: Option of integer := Map(O, function(V: integer): integer begin return V + 7 end);
  var Step2: Option of string := AndThen(Step1,
    function(V: integer): Option of string begin return Some(IntToStr(V)) end);
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
begin
  var R: Result of integer, string := Ok(6);
  var M: Result of integer, string := Std.Result.Map(R, function(V: integer): integer begin return V * 7 end);
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
begin
  var O: Option of integer := Some(10);
  var M: Option of string := Std.Option.AndThen(O,
    function(V: integer): Option of string begin return Some(IntToStr(V)) end);
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
begin
  var Factor: integer := 100;
  var R: Result of integer, string := Ok(3);
  var M: Result of integer, string := Map(R, function(V: integer): integer begin return V * Factor end);
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
begin
  var Prefix: string := 'val=';
  var O: Option of integer := Some(42);
  var M: Option of string := Map(O, function(V: integer): string begin return Prefix + IntToStr(V) end);
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
begin
  var R: Result of integer, string := Ok(2);
  var S1: Result of integer, string := Map(R, function(V: integer): integer begin return V + 3 end);
  var S2: Result of integer, string := Map(S1, function(V: integer): integer begin return V * 10 end);
  var S3: Result of string, string := AndThen(S2,
    function(V: integer): Result of string, string begin return Ok('result=' + IntToStr(V)) end);
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
begin
  var R: Result of integer, string := Error('boom');
  var S1: Result of integer, string := Map(R, function(V: integer): integer begin return V + 3 end);
  var S2: Result of integer, string := Map(S1, function(V: integer): integer begin return V * 10 end);
  var S3: Result of string, string := AndThen(S2,
    function(V: integer): Result of string, string begin return Ok(IntToStr(V)) end);
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
begin
  var O: Option of integer := None;
  var S1: Option of integer := Map(O, function(V: integer): integer begin return V + 1 end);
  var S2: Option of integer := Map(S1, function(V: integer): integer begin return V * 2 end);
  var S3: Option of string := AndThen(S2,
    function(V: integer): Option of string begin return Some(IntToStr(V)) end);
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
begin
  var R: Result of integer, string := Error('first');
  var M: Result of integer, string := OrElse(R,
    function(E: string): Result of integer, string begin return Error('replaced: ' + E) end);
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
begin
  var O: Option of integer := None;
  var M: Option of integer := OrElse(O,
    function(): Option of integer begin return None end);
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
begin
  var R: Result of integer, string := Ok(42);
  var M: Result of integer, string := Map(R, function(V: integer): integer begin return V end);
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
begin
  var O: Option of integer := Some(7);
  var M: Option of integer := Map(O, function(V: integer): integer begin return V end);
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
begin
  var R: Result of integer, string := Ok(5);
  var M: Result of Option of integer, string := Std.Result.Map(R,
    function(V: integer): Option of integer begin return Some(V * 2) end);
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
begin
  var O: Option of integer := Some(3);
  var M: Option of Result of integer, string := Std.Option.Map(O,
    function(V: integer): Result of integer, string begin return Ok(V * 4) end);
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

begin
  var R: Result of integer, string := ParsePositive('42');
  var Mapped: Result of Option of integer, string := Std.Result.Map(R,
    function(V: integer): Option of integer
    begin
      if V > 10 then return Some(V)
      else return None
    end);
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
begin
  var R: Result of integer, string := Ok(0);
  var M: Result of boolean, string := Map(R, function(V: integer): boolean begin return V > 0 end);
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
begin
  var O: Option of string := Some('123');
  var M: Option of integer := Map(O, function(S: string): integer begin return StrToInt(S) + 1 end);
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
begin
  var X: integer := 42;
  Std.Result.Map(X, function(V: integer): integer begin return V end)
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
begin
  var X: string := 'hello';
  Std.Option.Map(X, function(V: string): string begin return V end)
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
begin
  var X: boolean := true;
  Std.Result.AndThen(X, function(V: boolean): Result of boolean, string begin return Ok(V) end)
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
begin
  var X: integer := 1;
  Std.Option.AndThen(X, function(V: integer): Option of integer begin return Some(V) end)
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
begin
  var X: integer := 0;
  Std.Result.OrElse(X, function(E: integer): Result of integer, string begin return Ok(0) end)
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
begin
  var X: integer := 0;
  Std.Option.OrElse(X, function(): Option of integer begin return Some(0) end)
end.",
    )
    .message;
    assert!(
        msg.contains("must be an Option"),
        "expected type error, got: {msg}"
    );
}

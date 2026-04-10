use super::super::compile_err;
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

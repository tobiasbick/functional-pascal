/// Tests for functions returning `Result` — `docs/pascal/07-error-handling.md`.
use super::compile_and_run;

// ── Divide pattern from the docs ────────────────────────────────────────

#[test]
fn divide_returns_ok() {
    let out = compile_and_run(
        "program T;
function Divide(A: integer; B: integer): Result of integer, string;
begin
  if B = 0 then
    return Error('Division by zero')
  else
    return Ok(A div B)
end;
begin
  case Divide(10, 2) of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn divide_returns_error_on_zero() {
    let out = compile_and_run(
        "program T;
function Divide(A: integer; B: integer): Result of integer, string;
begin
  if B = 0 then
    return Error('Division by zero')
  else
    return Ok(A div B)
end;
begin
  case Divide(10, 0) of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["Division by zero"]);
}

// ── Error message is preserved through destructuring ────────────────────

#[test]
fn result_error_message_preserved_in_case() {
    let out = compile_and_run(
        "program T;
function Failing(): Result of integer, string;
begin
  return Error('detailed error info')
end;
begin
  case Failing() of
    Ok(V):    Std.Console.WriteLn('ok');
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["detailed error info"]);
}

// ── Result of string, string (different inner types) ────────────────────

#[test]
fn result_of_string_string() {
    let out = compile_and_run(
        "program T;
function Greet(Name: string): Result of string, string;
begin
  if Name = '' then
    return Error('empty name')
  else
    return Ok('Hello, ' + Name)
end;
begin
  case Greet('Alice') of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["Hello, Alice"]);
}

#[test]
fn result_of_string_string_error_branch() {
    let out = compile_and_run(
        "program T;
function Greet(Name: string): Result of string, string;
begin
  if Name = '' then
    return Error('empty name')
  else
    return Ok('Hello, ' + Name)
end;
begin
  case Greet('') of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["empty name"]);
}

// ── Result as function parameter ────────────────────────────────────────

#[test]
fn result_passed_as_function_parameter() {
    let out = compile_and_run(
        "program T;
function Describe(R: Result of integer, string): string;
begin
  case R of
    Ok(V):    return 'Got ' + Std.Conv.IntToStr(V);
    Error(E): return 'Err: ' + E
  end
end;
begin
  Std.Console.WriteLn(Describe(Ok(7)));
  Std.Console.WriteLn(Describe(Error('nope')))
end.",
    );
    assert_eq!(out.lines, vec!["Got 7", "Err: nope"]);
}

// ── Multiple conditional returns ────────────────────────────────────────

#[test]
fn result_multiple_error_conditions() {
    let out = compile_and_run(
        "program T;
function Validate(X: integer): Result of integer, string;
begin
  if X < 0 then
    return Error('negative');
  if X > 100 then
    return Error('too large');
  return Ok(X)
end;
begin
  case Validate(-5) of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end;
  case Validate(200) of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end;
  case Validate(50) of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["negative", "too large", "50"]);
}

use super::super::{compile_and_run, compile_err};
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

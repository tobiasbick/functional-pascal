/// Tests for basic function and procedure declarations.
///
/// **Documentation:** [docs/pascal/language/functions/README.md](docs/pascal/language/functions/README.md)
use super::super::*;

// ═══════════════════════════════════════════════════════════════
// RECURSION
// ═══════════════════════════════════════════════════════════════

#[test]
fn recursive_function() {
    let out = compile_and_run(
        "\
program RecurTest;

function Fact(n: integer): integer;
begin
  if n <= 1 then
    return 1
  else
    return n * Fact(n - 1)
end;

begin
  Std.Console.WriteLn(Fact(5))
end.",
    );
    assert_eq!(out.lines, vec!["120"]);
}

#[test]
fn recursive_function_base_case_zero() {
    let out = compile_and_run(
        "\
program RecurBase;
uses Std.Console;

function Fib(N: integer): integer;
begin
  if N <= 1 then
    return N
  else
    return Fib(N - 1) + Fib(N - 2)
end;

begin
  WriteLn(Fib(0));
  WriteLn(Fib(1));
  WriteLn(Fib(10))
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "55"]);
}

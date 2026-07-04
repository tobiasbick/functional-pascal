/// Tests for basic function and procedure declarations.
///
/// **Documentation:** [docs/pascal/language/functions/README.md](docs/pascal/language/functions/README.md)
use super::super::*;

// ═══════════════════════════════════════════════════════════════
// PROCEDURES
// ═══════════════════════════════════════════════════════════════

#[test]
fn procedure_call() {
    let out = compile_and_run(
        "\
program ProcTest;

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
fn procedure_multiple_params() {
    let out = compile_and_run(
        "\
program ProcMulti;
uses Std.Console;

procedure PrintSum(A: integer; B: integer; C: integer);
begin
  WriteLn(A + B + C)
end;

begin
  PrintSum(1, 2, 3)
end.",
    );
    assert_eq!(out.lines, vec!["6"]);
}

#[test]
fn procedure_zero_params() {
    let out = compile_and_run(
        "\
program ProcZero;
uses Std.Console;

procedure SayHi();
begin
  WriteLn('Hi')
end;

begin
  SayHi()
end.",
    );
    assert_eq!(out.lines, vec!["Hi"]);
}

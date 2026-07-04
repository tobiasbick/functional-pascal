/// Tests for basic function and procedure declarations.
///
/// **Documentation:** [docs/pascal/language/functions/README.md](docs/pascal/language/functions/README.md)
use super::super::*;

// ═══════════════════════════════════════════════════════════════
// BASIC FUNCTIONS
// ═══════════════════════════════════════════════════════════════

#[test]
fn simple_function() {
    let out = compile_and_run(
        "\
program FuncTest;

function Double(x: integer): integer;
begin
  return x * 2
end;

begin
  Std.Console.WriteLn(Double(21))
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn function_call_is_case_insensitive_at_runtime() {
    let out = compile_and_run(
        "\
program FuncCase;
uses Std.Console;

function Double(x: integer): integer;
begin
  return x * 2
end;

begin
  WriteLn(double(21))
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn function_returning_string() {
    let out = compile_and_run(
        "\
program FuncStr;
uses Std.Console;

function Greet(Name: string): string;
begin
  return 'Hello, ' + Name + '!'
end;

begin
  WriteLn(Greet('Alice'))
end.",
    );
    assert_eq!(out.lines, vec!["Hello, Alice!"]);
}

#[test]
fn function_returning_boolean() {
    let out = compile_and_run(
        "\
program FuncBool;
uses Std.Console;

function IsPositive(N: integer): boolean;
begin
  return N > 0
end;

begin
  WriteLn(IsPositive(5));
  WriteLn(IsPositive(-3));
  WriteLn(IsPositive(0))
end.",
    );
    assert_eq!(out.lines, vec!["true", "false", "false"]);
}

#[test]
fn function_returning_real() {
    let out = compile_and_run(
        "\
program FuncReal;
uses Std.Console;

function Half(N: real): real;
begin
  return N / 2.0
end;

begin
  WriteLn(Half(10.0))
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn function_zero_params() {
    let out = compile_and_run(
        "\
program FuncZero;
uses Std.Console;

function FortyTwo(): integer;
begin
  return 42
end;

begin
  WriteLn(FortyTwo())
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

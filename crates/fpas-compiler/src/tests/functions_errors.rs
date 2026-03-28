/// Negative tests for function and procedure declarations.
///
/// **Documentation:** [docs/pascal/04-functions.md](docs/pascal/04-functions.md)
use super::*;

// ═══════════════════════════════════════════════════════════════
// WRONG ARGUMENT COUNT
// ═══════════════════════════════════════════════════════════════

#[test]
fn too_few_arguments() {
    let err = compile_err(
        "\
program TooFew;

function Add(A: integer; B: integer): integer;
begin
  return A + B
end;

begin
  var R: integer := Add(1)
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT);
}

#[test]
fn too_many_arguments() {
    let err = compile_err(
        "\
program TooMany;

function Add(A: integer; B: integer): integer;
begin
  return A + B
end;

begin
  var R: integer := Add(1, 2, 3)
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT);
}

#[test]
fn zero_args_to_parameterized_function() {
    let err = compile_err(
        "\
program ZeroArgs;

function Inc(X: integer): integer;
begin
  return X + 1
end;

begin
  var R: integer := Inc()
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT);
}

#[test]
fn args_to_zero_param_function() {
    let err = compile_err(
        "\
program ArgsToZero;

function GetVal(): integer;
begin
  return 42
end;

begin
  var R: integer := GetVal(1)
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT);
}

#[test]
fn procedure_too_many_arguments() {
    let err = compile_err(
        "\
program ProcTooMany;

procedure Say(Msg: string);
begin
  Std.Console.WriteLn(Msg)
end;

begin
  Say('hi', 'extra')
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT);
}

// ═══════════════════════════════════════════════════════════════
// TYPE MISMATCH
// ═══════════════════════════════════════════════════════════════

#[test]
fn argument_type_mismatch() {
    let err = compile_err(
        "\
program ArgMismatch;

function Double(X: integer): integer;
begin
  return X * 2
end;

begin
  var R: integer := Double('hello')
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

#[test]
fn return_type_mismatch_in_assignment() {
    let err = compile_err(
        "\
program RetMismatch;

function GetName(): string;
begin
  return 'Alice'
end;

begin
  var N: integer := GetName()
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

#[test]
fn procedure_result_used_as_value() {
    let err = compile_err(
        "\
program ProcAsVal;

procedure DoNothing();
begin
end;

begin
  var R: integer := DoNothing()
end.",
    );
    assert!(
        err.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH
            || err.message.contains("procedure")
            || err.message.contains("void"),
        "unexpected error: {err:?}"
    );
}

// ═══════════════════════════════════════════════════════════════
// UNKNOWN FUNCTION
// ═══════════════════════════════════════════════════════════════

#[test]
fn call_undeclared_function() {
    let err = compile_err(
        "\
program UnknownFunc;

begin
  var R: integer := Nonexistent(42)
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_UNKNOWN_NAME);
}

// ═══════════════════════════════════════════════════════════════
// FUNCTION REDEFINITION — the last definition wins
// ═══════════════════════════════════════════════════════════════

#[test]
fn redefined_function_uses_last_definition() {
    let out = compile_and_run(
        "\
program RedefFunc;
uses Std.Console;

function Foo(): integer;
begin
  return 1
end;

function Foo(): integer;
begin
  return 2
end;

begin
  WriteLn(Foo())
end.",
    );
    assert_eq!(out.lines, vec!["2"]);
}

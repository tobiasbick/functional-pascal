use super::*;

#[test]
fn generic_identity_function_integer() {
    let out = compile_and_run(
        "\
program GenericIdentity;
uses Std.Console;

function Identity<T>(Value: T): T;
begin
  return Value
end;

begin
  WriteLn(Identity(42))
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn generic_identity_function_string() {
    let out = compile_and_run(
        "\
program GenericIdentityStr;
uses Std.Console;

function Identity<T>(Value: T): T;
begin
  return Value
end;

begin
  WriteLn(Identity('hello'))
end.",
    );
    assert_eq!(out.lines, vec!["hello"]);
}

#[test]
fn generic_identity_function_boolean() {
    let out = compile_and_run(
        "\
program GenericIdentityBool;
uses Std.Console;

function Identity<T>(Value: T): T;
begin
  return Value
end;

begin
  WriteLn(Identity(true))
end.",
    );
    assert_eq!(out.lines, vec!["true"]);
}

#[test]
fn generic_identity_function_real() {
    let out = compile_and_run(
        "\
program GenericIdentityReal;
uses Std.Console;

function Identity<T>(Value: T): T;
begin
  return Value
end;

begin
  WriteLn(Identity(3.14))
end.",
    );
    assert_eq!(out.lines, vec!["3.14"]);
}

#[test]
fn generic_function_called_multiple_times_different_types() {
    let out = compile_and_run(
        "\
program GenericMultiCall;
uses Std.Console, Std.Conv;

function Echo<T>(V: T): T;
begin
  return V
end;

begin
  WriteLn(IntToStr(Echo(1)));
  WriteLn(Echo('two'));
  WriteLn(Echo(true))
end.",
    );
    assert_eq!(out.lines, vec!["1", "two", "true"]);
}

#[test]
fn generic_function_nested_call() {
    let out = compile_and_run(
        "\
program GenericNested;
uses Std.Console;

function Echo<T>(V: T): T;
begin
  return V
end;

begin
  WriteLn(Echo(Echo(Echo('deep'))))
end.",
    );
    assert_eq!(out.lines, vec!["deep"]);
}

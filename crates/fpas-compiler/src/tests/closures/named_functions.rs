use super::*;

#[test]
fn named_function_as_value() {
    let out = compile_and_run(
        "\
program NamedFuncRef;
uses Std.Console;
function Double(X: integer): integer;
begin
  return X * 2
end;
function Apply(F: function(X: integer): integer; Value: integer): integer;
begin
  return F(Value)
end;
begin
  WriteLn(Apply(Double, 5))
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

#[test]
fn named_function_assigned_to_variable() {
    let out = compile_and_run(
        "\
program NamedAsVar;
uses Std.Console;

function Square(X: integer): integer;
begin
  return X * X
end;

begin
  var F: function(X: integer): integer := Square;
  WriteLn(F(6))
end.",
    );
    assert_eq!(out.lines, vec!["36"]);
}

#[test]
fn named_function_reassigned_variable() {
    let out = compile_and_run(
        "\
program FuncReassign;
uses Std.Console;

function Add1(X: integer): integer;
begin
  return X + 1
end;

function Mul2(X: integer): integer;
begin
  return X * 2
end;

begin
  mutable var F: function(X: integer): integer := Add1;
  WriteLn(F(5));
  F := Mul2;
  WriteLn(F(5))
end.",
    );
    assert_eq!(out.lines, vec!["6", "10"]);
}

#[test]
fn named_procedure_as_value() {
    let out = compile_and_run(
        "\
program ProcRef;
uses Std.Console;

procedure Shout(S: string);
begin
  WriteLn(S + '!')
end;

procedure Invoke(Action: procedure(S: string); Text: string);
begin
  Action(Text)
end;

begin
  Invoke(Shout, 'Hello')
end.",
    );
    assert_eq!(out.lines, vec!["Hello!"]);
}

use super::*;
#[test]
fn lambda_assigned_to_variable() {
    let out = compile_and_run(
        "\
program LambdaVar;
uses Std.Console;
begin
  var Square: function(X: integer): integer :=
    function(X: integer): integer
    begin
      return X * X
    end;
  WriteLn(Square(4))
end.",
    );
    assert_eq!(out.lines, vec!["16"]);
}
#[test]
fn lambda_passed_as_argument() {
    let out = compile_and_run(
        "\
program LambdaArg;
uses Std.Console;
function Apply(F: function(X: integer): integer; Value: integer): integer;
begin
  return F(Value)
end;
begin
  WriteLn(Apply(
    function(X: integer): integer
    begin
      return X * 3
    end,
    7))
end.",
    );
    assert_eq!(out.lines, vec!["21"]);
}
#[test]
fn lambda_with_multiple_params() {
    let out = compile_and_run(
        "\
program LambdaMultiParam;
uses Std.Console;
begin
  var Add: function(A: integer; B: integer): integer :=
    function(A: integer; B: integer): integer
    begin
      return A + B
    end;
  WriteLn(Add(3, 4))
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}
#[test]
fn lambda_returned_directly() {
    let out = compile_and_run(
        "\
program LambdaReturn;
uses Std.Console;
function GetCounter(): function(X: integer): integer;
begin
  return function(X: integer): integer begin return X + 1 end
end;
begin
  var Inc: function(X: integer): integer := GetCounter();
  WriteLn(Inc(41))
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

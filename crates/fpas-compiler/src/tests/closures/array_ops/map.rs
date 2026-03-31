use super::*;

#[test]
fn array_map_with_named_function() {
    let out = compile_and_run(
        "\
program MapNamed;
uses Std.Console, Std.Array;
function Square(X: integer): integer;
begin
  return X * X
end;
begin
  var Nums: array of integer := [1, 2, 3];
  var Squared: array of integer := Map(Nums, Square);
  for V: integer in Squared do
    Write(V);
  WriteLn('')
end.",
    );
    assert_eq!(out.lines, vec!["149"]);
}

#[test]
fn map_empty_array() {
    let out = compile_and_run(
        "\
program MapEmpty;
uses Std.Console, Std.Array;
function Double(X: integer): integer;
begin
  return X * 2
end;
begin
  var Empty: array of integer := [];
  var Res: array of integer := Map(Empty, Double);
  WriteLn(Length(Res))
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn map_with_named_function_stored_in_variable() {
    let out = compile_and_run(
        "\
program MapNamedVar;
uses Std.Console, Std.Array;
function Triple(X: integer): integer;
begin
  return X * 3
end;
begin
  var Nums: array of integer := [1, 2, 3];
  var F: function(X: integer): integer := Triple;
  var Tripled: array of integer := Map(Nums, F);
  for V: integer in Tripled do
    Write(V, ' ');
  WriteLn('')
end.",
    );
    assert_eq!(out.lines, vec!["3 6 9 "]);
}

#[test]
fn array_map_callback_can_spawn_and_wait_for_tasks() {
    let out = compile_and_run(
        "\
program MapAsync;
uses Std.Array, Std.Console, Std.Task;

function DoubleLater(X: integer): integer;
begin
  return X * 2
end;

function SpawnAndWait(X: integer): integer;
begin
  var Tsk: task := go DoubleLater(X);
  return Std.Task.Wait(Tsk)
end;

begin
  var Numbers: array of integer := [1, 2, 3];
  var Doubled: array of integer := Map(Numbers, SpawnAndWait);
  for V: integer in Doubled do
    Write(V, ' ');
  WriteLn('')
end.",
    );

    assert_eq!(out.lines, vec!["2 4 6 "]);
}

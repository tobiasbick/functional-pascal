//! Capturing closure integration tests.
//!
//! **Documentation:** [docs/pascal/language/functions/closures.md](docs/pascal/language/functions/closures.md)

use super::*;

#[test]
fn anonymous_procedure_literal_runs() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
begin
  var Print: procedure(Value: integer) :=
    procedure(Value: integer)
    begin
      WriteLn(IntToStr(Value))
    end;
  Print(7)
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

#[test]
fn immutable_local_capture_by_value() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
begin
  var Base: integer := 10;
  var AddBase: function(Value: integer): integer :=
    function(Value: integer): integer
    begin
      return Base + Value
    end;
  WriteLn(IntToStr(AddBase(5)))
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}

#[test]
fn returned_closure_outlives_creator() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
function Counter(): function(): integer;
begin
  mutable var Value: integer := 0;
  return function(): integer
  begin
    Value := Value + 1;
    return Value
  end
end;
begin
  var Next: function(): integer := Counter();
  WriteLn(IntToStr(Next()));
  WriteLn(IntToStr(Next()));
  WriteLn(IntToStr(Next()))
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "3"]);
}

#[test]
fn sibling_closures_share_mutable_cell() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
begin
  mutable var Count: integer := 0;
  var Inc: procedure() :=
    procedure()
    begin
      Count := Count + 1
    end;
  var Show: function(): integer :=
    function(): integer
    begin
      return Count
    end;
  Inc();
  Inc();
  WriteLn(IntToStr(Show()))
end.",
    );
    assert_eq!(out.lines, vec!["2"]);
}

#[test]
fn independent_environments_per_invocation() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
function Counter(): function(): integer;
begin
  mutable var Value: integer := 0;
  return function(): integer
  begin
    Value := Value + 1;
    return Value
  end
end;
begin
  var A: function(): integer := Counter();
  var B: function(): integer := Counter();
  WriteLn(IntToStr(A()));
  WriteLn(IntToStr(B()));
  WriteLn(IntToStr(A()))
end.",
    );
    assert_eq!(out.lines, vec!["1", "1", "2"]);
}

#[test]
fn nested_closure_recaptures_enclosing_capture() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
begin
  var Prefix: string := 'hi';
  var Make: function(): function(S: string): string :=
    function(): function(S: string): string
    begin
      return function(S: string): string
      begin
        return Prefix + S
      end
    end;
  var Join: function(S: string): string := Make();
  WriteLn(Join('!'))
end.",
    );
    assert_eq!(out.lines, vec!["hi!"]);
}

#[test]
fn nested_closure_propagates_non_root_capture() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
function MakeOuter(Base: integer): function(): function(): integer;
begin
  return function(): function(): integer
  begin
    return function(): integer
    begin
      return Base
    end
  end
end;
begin
  var Outer: function(): function(): integer := MakeOuter(9);
  var Inner: function(): integer := Outer();
  WriteLn(IntToStr(Inner()))
end.",
    );
    assert_eq!(out.lines, vec!["9"]);
}

#[test]
fn mutable_callable_capture_is_dereferenced_before_call() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
function MakeCaller(): function(): integer;
begin
  mutable var Target: function(): integer :=
    function(): integer
    begin
      return 4
    end;
  return function(): integer
  begin
    return Target()
  end
end;
begin
  var Call: function(): integer := MakeCaller();
  WriteLn(IntToStr(Call()))
end.",
    );
    assert_eq!(out.lines, vec!["4"]);
}

#[test]
fn named_nested_routine_escapes_as_closure() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
function MakeAdder(Base: integer): function(Value: integer): integer;
  function Add(Value: integer): integer;
  begin
    return Base + Value
  end;
begin
  return Add
end;
begin
  var AddTen: function(Value: integer): integer := MakeAdder(10);
  WriteLn(IntToStr(AddTen(5)))
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}

#[test]
fn capture_string_array_and_record() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type Point = record
  X: integer;
  Y: integer;
end;
begin
  var Label: string := 'pt';
  var Nums: array of integer := [1, 2];
  var P: Point := record X := 3; Y := 4; end;
  var Show: procedure() :=
    procedure()
    begin
      WriteLn(Label);
      WriteLn(IntToStr(Nums[0]));
      WriteLn(IntToStr(P.X + P.Y))
    end;
  Show()
end.",
    );
    assert_eq!(out.lines, vec!["pt", "1", "7"]);
}

#[test]
fn mutable_array_capture_keeps_cell_after_index_assignment() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
function MakeCounter(): function(): integer;
begin
  mutable var Values: array of integer := [0];
  return function(): integer
  begin
    Values[0] := Values[0] + 1;
    return Values[0]
  end
end;
begin
  var Next: function(): integer := MakeCounter();
  WriteLn(IntToStr(Next()));
  WriteLn(IntToStr(Next()))
end.",
    );
    assert_eq!(out.lines, vec!["1", "2"]);
}

#[test]
fn mutable_record_capture_keeps_cell_after_field_assignment() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type Counter = record
  Value: integer;
end;
function MakeCounter(): function(): integer;
begin
  mutable var State: Counter := record Value := 0; end;
  return function(): integer
  begin
    State.Value := State.Value + 1;
    return State.Value
  end
end;
begin
  var Next: function(): integer := MakeCounter();
  WriteLn(IntToStr(Next()));
  WriteLn(IntToStr(Next()))
end.",
    );
    assert_eq!(out.lines, vec!["1", "2"]);
}

#[test]
fn immutable_closure_may_cross_task_boundary() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv, Std.Task;
begin
  var N: integer := 3;
  var Work: function(): integer :=
    function(): integer
    begin
      return N * 2
    end;
  var Handle: task := go Work();
  WriteLn(IntToStr(Wait(Handle)))
end.",
    );
    assert_eq!(out.lines, vec!["6"]);
}

#[test]
fn mutable_closure_rejected_across_task_boundary() {
    let err = compile_err(
        "program T;
uses Std.Task;
begin
  mutable var Count: integer := 0;
  var Inc: procedure() :=
    procedure()
    begin
      Count := Count + 1
    end;
  go Inc()
end.",
    );
    assert!(
        err.message.contains("task-bound") || err.message.contains("Task-bound"),
        "expected task-bound diagnostic, got: {}",
        err.message
    );
}

#[test]
fn wrong_closure_parameter_type_is_rejected() {
    let err = compile_err(
        "program T;
begin
  var F: function(X: integer): integer :=
    function(X: string): integer
    begin
      return 1
    end
end.",
    );
    assert!(
        err.message.to_ascii_lowercase().contains("type")
            || err.message.to_ascii_lowercase().contains("mismatch"),
        "expected type mismatch, got: {}",
        err.message
    );
}

/// Tests for array types (creation, indexing, empty, string arrays).
///
/// **Documentation:** [docs/pascal/language/types/arrays.md](docs/pascal/language/types/arrays.md)
use super::*;

// ═══════════════════════════════════════════════════════════════
// POSITIVE — array basics (extending expressions.rs coverage)
// ═══════════════════════════════════════════════════════════════

#[test]
fn empty_array() {
    let out = compile_and_run(
        "\
program EmptyArr;
uses Std.Console, Std.Array, Std.Conv;
begin
  var A: array of integer := [];
  WriteLn(IntToStr(Std.Array.Length(A)))
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn array_of_strings() {
    let out = compile_and_run(
        "\
program ArrStrings;
uses Std.Console;
begin
  var Names: array of string := ['Alice', 'Bob', 'Carol'];
  WriteLn(Names[0]);
  WriteLn(Names[1]);
  WriteLn(Names[2])
end.",
    );
    assert_eq!(out.lines, vec!["Alice", "Bob", "Carol"]);
}

#[test]
fn array_of_booleans() {
    let out = compile_and_run(
        "\
program ArrBool;
uses Std.Console;
begin
  var Flags: array of boolean := [true, false, true];
  WriteLn(Flags[0]);
  WriteLn(Flags[1])
end.",
    );
    assert_eq!(out.lines, vec!["true", "false"]);
}

#[test]
fn array_passed_to_function() {
    let out = compile_and_run(
        "\
program ArrParam;
uses Std.Console, Std.Array, Std.Conv;

function SumAll(Items: array of integer): integer;
begin
  mutable var Total: integer := 0;
  for I: integer := 0 to Std.Array.Length(Items) - 1 do
    Total := Total + Items[I];
  return Total
end;

begin
  WriteLn(SumAll([10, 20, 30]))
end.",
    );
    assert_eq!(out.lines, vec!["60"]);
}

#[test]
fn array_returned_from_function() {
    let out = compile_and_run(
        "\
program ArrReturn;
uses Std.Console;

function MakeArray(): array of integer;
begin
  return [1, 2, 3]
end;

begin
  var A: array of integer := MakeArray();
  WriteLn(A[0]);
  WriteLn(A[2])
end.",
    );
    assert_eq!(out.lines, vec!["1", "3"]);
}

#[test]
fn array_push_and_length() {
    let out = compile_and_run(
        "\
program ArrPush;
uses Std.Console, Std.Array, Std.Conv;
begin
  mutable var Items: array of integer := [1, 2];
  Std.Array.Push(Items, 3);
  WriteLn(IntToStr(Std.Array.Length(Items)));
  WriteLn(Items[2])
end.",
    );
    assert_eq!(out.lines, vec!["3", "3"]);
}

#[test]
fn in_operator_checks_array_membership() {
    let out = compile_and_run(
        "\
  program ArrayInOperator;
  uses Std.Console;
  begin
    WriteLn(2 in [1, 2, 3]);
    WriteLn(5 in [1, 2, 3])
  end.",
    );
    assert_eq!(out.lines, vec!["true", "false"]);
}

#[test]
fn in_operator_checks_string_membership() {
    let out = compile_and_run(
        "\
  program StringInOperator;
  uses Std.Console;
  begin
    WriteLn('a' in 'pascal');
    WriteLn('asc' in 'pascal');
    WriteLn('z' in 'pascal')
  end.",
    );
    assert_eq!(out.lines, vec!["true", "true", "false"]);
}

#[test]
fn in_operator_rejects_incompatible_array_member_type() {
    let err = compile_err(
        "\
  program BadArrayInOperator;
  begin
    var Found: boolean := 'x' in [1, 2, 3]
  end.",
    );
    assert!(
        err.message.contains("Operator `in` requires"),
        "expected `in` type diagnostic, got: {err:#?}"
    );
}

// ═══════════════════════════════════════════════════════════════
// NEGATIVE — runtime errors
// ═══════════════════════════════════════════════════════════════

#[test]
fn array_index_out_of_bounds() {
    let msg = compile_run_err(
        "\
program ArrOob;
begin
  var A: array of integer := [10, 20, 30];
  Std.Console.WriteLn(A[5])
end.",
    );
    assert!(
        msg.to_lowercase().contains("index") || msg.to_lowercase().contains("bound"),
        "expected index-out-of-bounds error, got: {}",
        msg
    );
}

#[test]
fn array_negative_index() {
    let msg = compile_run_err(
        "\
program ArrNeg;
begin
  var A: array of integer := [10, 20];
  Std.Console.WriteLn(A[-1])
end.",
    );
    assert!(
        msg.to_lowercase().contains("index")
            || msg.to_lowercase().contains("bound")
            || msg.to_lowercase().contains("negative"),
        "expected index error, got: {}",
        msg
    );
}

// ═══════════════════════════════════════════════════════════════
// EDGE CASES — arrays of composite types
// ═══════════════════════════════════════════════════════════════

#[test]
fn array_of_records() {
    let out = compile_and_run(
        "\
program ArrRec;
uses Std.Console;
type Point = record X: integer; Y: integer; end;
begin
  var Pts: array of Point := [
    record X := 1; Y := 2; end,
    record X := 3; Y := 4; end
  ];
  WriteLn(Pts[0].X);
  WriteLn(Pts[1].Y)
end.",
    );
    assert_eq!(out.lines, vec!["1", "4"]);
}

#[test]
fn array_of_enums() {
    let out = compile_and_run(
        "\
program ArrEnum;
uses Std.Console;
type PaintColor = enum Red; Green; Blue; end;
begin
  var PaintColors: array of PaintColor := [PaintColor.Red, PaintColor.Blue, PaintColor.Green];
  if PaintColors[0] = PaintColor.Red then
    WriteLn('red')
  else
    WriteLn('other');
  if PaintColors[1] = PaintColor.Blue then
    WriteLn('blue')
  else
    WriteLn('other')
end.",
    );
    assert_eq!(out.lines, vec!["red", "blue"]);
}

#[test]
fn global_nested_array_assignment_uses_global_index_set() {
    let source = "\
program GlobalNestedArray;
uses Std.Console;
mutable var Grid: array of array of integer := [[1, 2], [3, 4]];
begin
  Grid[1][0] := 9;
  WriteLn(Grid[1][0])
end.";

    let chunk = compile_ok(source);
    assert!(
        chunk
            .code()
            .iter()
            .any(|op| matches!(op, fpas_bytecode::Op::GlobalIndexSet(_, 2))),
        "nested global array assignment should use GlobalIndexSet"
    );
    assert_eq!(compile_and_run(source).lines, vec!["9"]);
}

#[test]
fn array_of_enum_data_variants() {
    let out = compile_and_run(
        "\
program ArrEnumData;
uses Std.Console;
type Shape = enum Circle(Radius: real); Dot; end;
begin
  var Shapes: array of Shape := [Shape.Circle(5.0), Shape.Dot, Shape.Circle(1.0)];
  case Shapes[0] of
    Shape.Circle(R): WriteLn(R);
    Shape.Dot: WriteLn('dot')
  end;
  case Shapes[1] of
    Shape.Circle(R): WriteLn(R);
    Shape.Dot: WriteLn('dot')
  end
end.",
    );
    assert_eq!(out.lines, vec!["5", "dot"]);
}

#[test]
fn push_on_immutable_array_is_rejected() {
    let err = compile_err(
        "\
program PushImmut;
uses Std.Array;
begin
  var A: array of integer := [1, 2];
  Std.Array.Push(A, 3)
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_IMMUTABLE_ASSIGNMENT);
}

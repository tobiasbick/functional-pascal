/// Tests for array types (creation, indexing, empty, string arrays).
///
/// **Documentation:** [docs/pascal/05-types.md](docs/pascal/05-types.md)
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

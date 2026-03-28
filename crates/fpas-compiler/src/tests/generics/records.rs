use super::*;

// ═══════════════════════════════════════════════════════════════
// GENERIC RECORDS — positive
// ═══════════════════════════════════════════════════════════════

#[test]
fn generic_record_pair() {
    let out = compile_and_run(
        "\
program GenericPair;
uses Std.Console, Std.Conv;

type
  Pair<A, B> = record
    First: A;
    Second: B;
  end;

begin
  var P: Pair of integer, string := record
    First := 42;
    Second := 'hello';
  end;
  WriteLn(IntToStr(P.First) + ' ' + P.Second)
end.",
    );
    assert_eq!(out.lines, vec!["42 hello"]);
}

#[test]
fn generic_record_single_param() {
    let out = compile_and_run(
        "\
program GenericBox;
uses Std.Console;

type
  Box<T> = record
    Value: T;
  end;

begin
  var B: Box of string := record Value := 'inside'; end;
  WriteLn(B.Value)
end.",
    );
    assert_eq!(out.lines, vec!["inside"]);
}

#[test]
fn generic_record_with_integer() {
    let out = compile_and_run(
        "\
program GenericBoxInt;
uses Std.Console;

type
  Box<T> = record
    Value: T;
  end;

begin
  var B: Box of integer := record Value := 99; end;
  WriteLn(B.Value)
end.",
    );
    assert_eq!(out.lines, vec!["99"]);
}

#[test]
fn generic_record_nested_generic_field() {
    let out = compile_and_run(
        "\
program NestedGenericField;
uses Std.Console;

type
  Box<T> = record
    Value: T;
  end;

begin
  var B: Box of array of integer := record Value := [1, 2, 3]; end;
  WriteLn(B.Value[0])
end.",
    );
    assert_eq!(out.lines, vec!["1"]);
}

#[test]
fn generic_record_two_instances_different_types() {
    let out = compile_and_run(
        "\
program TwoBoxes;
uses Std.Console;

type
  Box<T> = record
    Value: T;
  end;

begin
  var A: Box of integer := record Value := 42; end;
  var B: Box of string := record Value := 'hi'; end;
  WriteLn(A.Value);
  WriteLn(B.Value)
end.",
    );
    assert_eq!(out.lines, vec!["42", "hi"]);
}

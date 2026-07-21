//! Record property integration tests.
//!
//! **Documentation:** [docs/pascal/language/types/record-properties.md](docs/pascal/language/types/record-properties.md)

use super::*;

#[test]
fn property_read_calls_getter() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type
  Box = record
    Value: integer;
    function GetValue(Self: Box): integer;
    begin
      return Self.Value
    end;
    property ValueProp: integer read GetValue;
  end;
begin
  var B: Box := record Value := 7; end;
  WriteLn(IntToStr(B.ValueProp))
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

#[test]
fn property_write_calls_setter_on_immutable_binding() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type
  Counter = record
    Value: integer;
    function GetValue(Self: Counter): integer;
    begin
      return Self.Value
    end;
    procedure SetValue(Self: Counter; V: integer);
    begin
      { Setter receives Self by value; for this test we only print. }
      WriteLn(IntToStr(V))
    end;
    property ValueProp: integer read GetValue write SetValue;
  end;
begin
  var C: Counter := record Value := 0; end;
  C.ValueProp := 42;
  WriteLn(IntToStr(C.ValueProp))
end.",
    );
    assert_eq!(out.lines, vec!["42", "0"]);
}

#[test]
fn property_read_from_postfix_chain() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type
  Box = record
    Value: integer;
    function GetValue(Self: Box): integer;
    begin
      return Self.Value
    end;
    property ValueProp: integer read GetValue;
    static function Make(Value: integer): Box;
    begin
      return record Value := Value; end
    end;
  end;
begin
  WriteLn(IntToStr(Box.Make(9).ValueProp))
end.",
    );
    assert_eq!(out.lines, vec!["9"]);
}

#[test]
fn rejects_read_only_property_write_at_compile_time() {
    let err = compile_err(
        "program T;
type
  Box = record
    function GetWidth(Self: Box): integer;
    begin
      return 0
    end;
    property Width: integer read GetWidth;
  end;
begin
  var B: Box := record end;
  B.Width := 1
end.",
    );
    assert!(
        err.message.contains("read-only") || err.message.contains("Width"),
        "{}",
        err.message
    );
}

#[test]
fn property_read_can_continue_through_fields_and_properties() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type
  Inner = record
    Value: integer;
    function GetValue(Self: Inner): integer;
    begin
      return Self.Value
    end;
    property Number: integer read GetValue;
  end;
  Outer = record
    Item: Inner;
    function GetItem(Self: Outer): Inner;
    begin
      return Self.Item
    end;
    property Child: Inner read GetItem;
  end;
begin
  var O: Outer := record Item := record Value := 17; end; end;
  WriteLn(IntToStr(O.Child.Value));
  WriteLn(IntToStr(O.Child.Number))
end.",
    );
    assert_eq!(out.lines, vec!["17", "17"]);
}

#[test]
fn nested_property_write_evaluates_receiver_then_value_once() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type
  Inner = record
    Id: integer;
    procedure SetValue(Self: Inner; Value: integer);
    begin
      WriteLn('setter:' + IntToStr(Value))
    end;
    property Value: integer write SetValue;
  end;
  Outer = record
    Item: Inner;
    function GetItem(Self: Outer): Inner;
    begin
      WriteLn('receiver');
      return Self.Item
    end;
    property Child: Inner read GetItem;
  end;
function BuildValue(): integer;
begin
  WriteLn('value');
  return 23
end;
begin
  var O: Outer := record Item := record Id := 1; end; end;
  O.Child.Value := BuildValue()
end.",
    );
    assert_eq!(out.lines, vec!["receiver", "value", "setter:23"]);
}

#[test]
fn property_result_can_receive_expression_and_statement_method_calls() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type
  Inner = record
    Value: integer;
    function Doubled(Self: Inner): integer;
    begin
      return Self.Value * 2
    end;
    procedure Print(Self: Inner);
    begin
      WriteLn('print:' + IntToStr(Self.Value))
    end;
  end;
  Outer = record
    Item: Inner;
    function GetItem(Self: Outer): Inner;
    begin
      return Self.Item
    end;
    property Child: Inner read GetItem;
  end;
begin
  var O: Outer := record Item := record Value := 6; end; end;
  WriteLn(IntToStr(O.Child.Doubled()));
  O.Child.Print()
end.",
    );
    assert_eq!(out.lines, vec!["12", "print:6"]);
}

#[test]
fn type_alias_exposes_record_property() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type
  Box = record
    Value: integer;
    function GetValue(Self: Box): integer;
    begin
      return Self.Value
    end;
    property Number: integer read GetValue;
  end;
  BoxAlias = Box;
begin
  var B: BoxAlias := record Value := 31; end;
  WriteLn(IntToStr(B.Number))
end.",
    );
    assert_eq!(out.lines, vec!["31"]);
}

#[test]
fn go_method_call_evaluates_property_receiver() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv, Std.Task;
type
  Inner = record
    Value: integer;
    function Doubled(Self: Inner): integer;
    begin
      return Self.Value * 2
    end;
  end;
  Outer = record
    Item: Inner;
    function GetItem(Self: Outer): Inner;
    begin
      return Self.Item
    end;
    property Child: Inner read GetItem;
  end;
begin
  var O: Outer := record Item := record Value := 8; end; end;
  var Tsk: task := go O.Child.Doubled();
  WriteLn(IntToStr(Std.Task.Wait(Tsk)))
end.",
    );
    assert_eq!(out.lines, vec!["16"]);
}

#[test]
fn computed_property_can_be_used_in_arithmetic() {
    let out = compile_and_run(
        "program T;
uses Std.Console;
type
  Selector = record
    Value: integer;
    function ReadSelected(Self: Selector): integer;
    begin
      return Self.Value
    end;
    property Selected: integer read ReadSelected;
    function Previous(Self: Selector): integer;
    begin
      return Self.Selected - 1
    end;
  end;
begin
  var S: Selector := record Value := 3; end;
  WriteLn(S.Previous())
end.",
    );
    assert_eq!(out.lines, vec!["2"]);
}

#[test]
fn method_call_on_property_can_be_used_in_boolean_expression() {
    let out = compile_and_run(
        "program T;
uses Std.Console;
type
  Child = record
    Alive: boolean;
    function IsAlive(Self: Child): boolean;
    begin
      return Self.Alive
    end;
  end;
  Parent = record
    Current: boolean;
    ChildValue: Child;
    function ReadChild(Self: Parent): Child;
    begin
      return Self.ChildValue
    end;
    property Custom: Child read ReadChild;
    function IsAlive(Self: Parent): boolean;
    begin
      return Self.Current and Self.Custom.IsAlive()
    end;
  end;
begin
  var P: Parent := record
    Current := true;
    ChildValue := record Alive := true; end;
  end;
  WriteLn(P.IsAlive())
end.",
    );
    assert_eq!(out.lines, vec!["true"]);
}

//! Record event integration tests.
//!
//! **Documentation:** [docs/pascal/language/types/record-events.md](docs/pascal/language/types/record-events.md)

use super::*;

#[test]
fn event_assign_assigned_raise_and_clear() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;

mutable var Slot: Option of procedure(Sender: integer) := None;

type
  Button = record
    Id: integer;

    function ReadOnClick(Self: Button): Option of procedure(Sender: integer);
    begin
      return Slot
    end;

    procedure WriteOnClick(Self: Button; Handler: Option of procedure(Sender: integer));
    begin
      Slot := Handler
    end;

    event OnClick: procedure(Sender: integer) read ReadOnClick write WriteOnClick;

    procedure RaiseClick(Self: Button);
    begin
      if Assigned(Self.OnClick) then
        Self.OnClick(Self.Id)
    end;
  end;

procedure Handle(Sender: integer);
begin
  WriteLn(IntToStr(Sender))
end;

begin
  var B: Button := record Id := 7; end;
  if Assigned(B.OnClick) then
    WriteLn('unexpected');
  B.OnClick := Handle;
  if Assigned(B.OnClick) then
    B.RaiseClick();
  B.OnClick := (nil);
  if not Assigned(B.OnClick) then
    WriteLn('cleared')
end.",
    );
    assert_eq!(out.lines, vec!["7", "cleared"]);
}

#[test]
fn event_accepts_closure_handler() {
    let out = compile_and_run(
        "program T;
uses Std.Console;

mutable var Slot: Option of procedure() := None;
mutable var Flag: boolean := false;

type
  Button = record
    function ReadOnClick(Self: Button): Option of procedure();
    begin
      return Slot
    end;
    procedure WriteOnClick(Self: Button; Handler: Option of procedure());
    begin
      Slot := Handler
    end;
    event OnClick: procedure() read ReadOnClick write WriteOnClick;
    procedure RaiseClick(Self: Button);
    begin
      if Assigned(Self.OnClick) then
        Self.OnClick()
    end;
  end;

begin
  var B: Button := record end;
  B.OnClick := procedure() begin Flag := true end;
  B.RaiseClick();
  if Flag then
    WriteLn('ok')
end.",
    );
    assert_eq!(out.lines, vec!["ok"]);
}

#[test]
fn function_event_returns_handler_result() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
mutable var Slot: Option of function(Value: integer): integer := None;
type
  Calculator = record
    function ReadCalculate(Self: Calculator): Option of function(Value: integer): integer;
    begin
      return Slot
    end;
    procedure WriteCalculate(
      Self: Calculator;
      Handler: Option of function(Value: integer): integer
    );
    begin
      Slot := Handler
    end;
    event OnCalculate: function(Value: integer): integer read ReadCalculate write WriteCalculate;
    function Calculate(Self: Calculator; Value: integer): integer;
    begin
      return Self.OnCalculate(Value)
    end;
  end;
function AddThree(Value: integer): integer;
begin
  return Value + 3
end;
begin
  var C: Calculator := record end;
  C.OnCalculate := AddThree;
  WriteLn(IntToStr(C.Calculate(7)))
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

#[test]
fn event_accepts_bound_method_and_replacement() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
mutable var Slot: Option of function(Value: integer): integer := None;
type
  Controller = record
    Base: integer;
    function Add(Self: Controller; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
  end;
  Source = record
    function ReadValue(Self: Source): Option of function(Value: integer): integer;
    begin
      return Slot
    end;
    procedure WriteValue(
      Self: Source;
      Handler: Option of function(Value: integer): integer
    );
    begin
      Slot := Handler
    end;
    event OnValue: function(Value: integer): integer read ReadValue write WriteValue;
    function RaiseValue(Self: Source; Value: integer): integer;
    begin
      return Self.OnValue(Value)
    end;
  end;
function Double(Value: integer): integer;
begin
  return Value * 2
end;
begin
  var S: Source := record end;
  var C: Controller := record Base := 10; end;
  S.OnValue := Double;
  S.OnValue := C.Add;
  WriteLn(IntToStr(S.RaiseValue(5)))
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}

#[test]
fn event_paths_through_property_evaluate_receiver_once_per_operation() {
    let out = compile_and_run(
        "program T;
uses Std.Console;
mutable var Slot: Option of procedure() := None;
type
  Button = record
    function ReadOnClick(Self: Button): Option of procedure();
    begin
      return Slot
    end;
    procedure WriteOnClick(Self: Button; Handler: Option of procedure());
    begin
      Slot := Handler
    end;
    event OnClick: procedure() read ReadOnClick write WriteOnClick;
  end;
  Form = record
    Child: Button;
    function GetButton(Self: Form): Button;
    begin
      WriteLn('receiver');
      return Self.Child
    end;
    property AcceptButton: Button read GetButton;
  end;
procedure Handle();
begin
  WriteLn('handled')
end;
begin
  var F: Form := record Child := record end; end;
  F.AcceptButton.OnClick := Handle;
  if Assigned(F.AcceptButton.OnClick) then
    F.AcceptButton.OnClick()
end.",
    );
    assert_eq!(
        out.lines,
        vec!["receiver", "receiver", "receiver", "handled"]
    );
}

#[test]
fn raising_empty_event_reports_runtime_error() {
    let message = compile_run_err(
        "program T;
mutable var Slot: Option of procedure() := None;
type
  Button = record
    function ReadOnClick(Self: Button): Option of procedure();
    begin
      return Slot
    end;
    procedure WriteOnClick(Self: Button; Handler: Option of procedure());
    begin
      Slot := Handler
    end;
    event OnClick: procedure() read ReadOnClick write WriteOnClick;
    procedure Click(Self: Button);
    begin
      Self.OnClick()
    end;
  end;
begin
  var B: Button := record end;
  B.Click()
end.",
    );
    assert!(message.contains("None"), "{message}");
}

#[test]
fn type_alias_exposes_event() {
    let out = compile_and_run(
        "program T;
uses Std.Console;
mutable var Slot: Option of procedure() := None;
type
  Button = record
    function ReadOnClick(Self: Button): Option of procedure();
    begin
      return Slot
    end;
    procedure WriteOnClick(Self: Button; Handler: Option of procedure());
    begin
      Slot := Handler
    end;
    event OnClick: procedure() read ReadOnClick write WriteOnClick;
    procedure Click(Self: Button);
    begin
      Self.OnClick()
    end;
  end;
  ButtonAlias = Button;
procedure Handle();
begin
  WriteLn('alias')
end;
begin
  var B: ButtonAlias := record end;
  B.OnClick := Handle;
  B.Click()
end.",
    );
    assert_eq!(out.lines, vec!["alias"]);
}

#[test]
fn event_retains_capturing_handler_after_creator_returns() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
mutable var Slot: Option of procedure() := None;
type
  Button = record
    function ReadOnClick(Self: Button): Option of procedure();
    begin
      return Slot
    end;
    procedure WriteOnClick(Self: Button; Handler: Option of procedure());
    begin
      Slot := Handler
    end;
    event OnClick: procedure() read ReadOnClick write WriteOnClick;
    procedure Click(Self: Button);
    begin
      Self.OnClick()
    end;
  end;
procedure Configure(ButtonValue: Button);
begin
  mutable var Count: integer := 0;
  ButtonValue.OnClick := procedure()
  begin
    Count := Count + 1;
    WriteLn(IntToStr(Count))
  end
end;
begin
  var B: Button := record end;
  Configure(B);
  B.Click();
  B.Click()
end.",
    );
    assert_eq!(out.lines, vec!["1", "2"]);
}

#[test]
fn event_handler_can_clear_itself_during_invocation() {
    let out = compile_and_run(
        "program T;
uses Std.Console;
mutable var Slot: Option of procedure(Sender: integer) := None;
type
  Button = record
    Id: integer;
    function ReadOnClick(Self: Button): Option of procedure(Sender: integer);
    begin
      return Slot
    end;
    procedure WriteOnClick(Self: Button; Handler: Option of procedure(Sender: integer));
    begin
      Slot := Handler
    end;
    event OnClick: procedure(Sender: integer) read ReadOnClick write WriteOnClick;
    procedure Click(Self: Button);
    begin
      Self.OnClick(Self.Id)
    end;
  end;
mutable var Target: Button := record Id := 1; end;
procedure Handle(Sender: integer);
begin
  Target.OnClick := nil;
  WriteLn('handled')
end;
begin
  Target.OnClick := Handle;
  Target.Click();
  if not Assigned(Target.OnClick) then
    WriteLn('cleared')
end.",
    );
    assert_eq!(out.lines, vec!["handled", "cleared"]);
}

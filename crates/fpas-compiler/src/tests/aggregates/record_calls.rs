//! Record data participates in ordinary calls, closures, and optional handlers.

use super::*;

#[test]
fn explicit_record_parameters_and_optional_procedures_execute() {
    assert_succeeds(
        r#"program RecordCalls;
var LastValue: integer := 0;
type Counter = record Value: integer; end record;
type Button = record OnValue: Option of (procedure(Value: integer)) := Option.None; end record;
function CounterDouble(Receiver: Counter): integer;
begin return Receiver.Value * 2; end function;
function CounterRead(Receiver: Counter): integer;
begin return Receiver.Value; end function;
procedure CounterWrite(Receiver: Counter; Value: integer);
begin LastValue := Receiver.Value + Value; end procedure;
procedure Remember(Value: integer);
begin LastValue := Value; end procedure;
procedure Invoke(Receiver: Button; Value: integer);
begin
  case Receiver.OnValue of
    when Option.Some(const Handler): Handler(Value);
    when Option.None: null;
  end case;
end procedure;
begin
  const C := Counter(Value := 6);
  if CounterDouble(C) <> 12 then panic('record argument'); end if;
  if CounterRead(C) <> 6 then panic('read'); end if;
  CounterWrite(C, 3);
  if LastValue <> 9 then panic('write'); end if;
  var B := Button();
  Invoke(B, 99);
  if LastValue <> 9 then panic('absent handler ran'); end if;
  B.OnValue := Option.Some(Remember);
  Invoke(B, 17);
  if LastValue <> 17 then panic('stored handler'); end if;
  B.OnValue := Option.None;
  Invoke(B, 99);
  if LastValue <> 17 then panic('cleared handler ran'); end if;
end program;"#,
    );
}

#[test]
fn ordinary_factories_and_generic_functions_use_explicit_record_arguments() {
    assert_succeeds(
        r#"program GenericRecordCalls;
type Box = record Value: integer; end record;
function BoxCreate(Value: integer): Box;
begin return Box(Value := Value); end function;
function BoxRead(Receiver: Box): integer;
begin return Receiver.Value; end function;
function BoxMap of (T)(Receiver: Box; Transform: function(Value: integer): T): T;
begin return Transform(Receiver.Value); end function;
function Double(Value: integer): integer;
begin return Value * 2; end function;
begin
  const B := boxcreate(11);
  if BoxMap(B, Double) <> 22 then panic('generic call'); end if;
  if BoxRead(BoxCreate(7)) <> 7 then panic('factory call'); end if;
end program;"#,
    );
}

#[test]
fn ordinary_closures_capture_an_explicit_record_snapshot() {
    assert_succeeds(
        r#"program RecordSnapshot;
type Counter = record Base: integer; end record;
function CounterAdd(Receiver: Counter; Value: integer): integer;
begin return Receiver.Base + Value; end function;
begin
  var C := Counter(Base := 10);
  const Snapshot := C;
  const AddToCounter := function(Value: integer): integer
    begin return CounterAdd(Snapshot, Value); end function;
  C.Base := 99;
  if AddToCounter(7) <> 17 then panic('record snapshot'); end if;
end program;"#,
    );
}

#[test]
fn optional_function_fields_preserve_captured_record_values() {
    assert_succeeds(
        r#"program OptionalFunction;
type Counter = record Base: integer; end record;
type Handler = function(Value: integer): integer;
type Source = record OnValue: Option of (Handler) := Option.None; end record;
function CounterAdd(Receiver: Counter; Value: integer): integer;
begin return Receiver.Base + Value; end function;
begin
  const C := Counter(Base := 12);
  var S := Source();
  S.OnValue := Option.Some(function(Value: integer): integer
    begin return CounterAdd(C, Value); end function);
  case S.OnValue of
    when Option.Some(const Apply):
      if Apply(8) <> 20 then panic('captured handler'); end if;
    when Option.None: panic('missing handler');
  end case;
end program;"#,
    );
}

#[test]
fn nested_record_calls_evaluate_arguments_once_in_written_order() {
    assert_succeeds(
        r#"program RecordCallOrder;
var Step: integer := 0;
var Written: integer := 0;
type Inner = record Value: integer; end record;
type Outer = record Item: Inner; end record;
function InnerRead(Receiver: Inner): integer;
begin Step := Step * 10 + 4; return Receiver.Value; end function;
procedure InnerWrite(Receiver: Inner; Value: integer);
begin Step := Step * 10 + 3; Written := Value; end procedure;
function OuterRead(Receiver: Outer): Inner;
begin Step := Step * 10 + 1; return Receiver.Item; end function;
function BuildValue(): integer;
begin Step := Step * 10 + 2; return 23; end function;
begin
  const O := Outer(Item := Inner(Value := 17));
  InnerWrite(OuterRead(O), BuildValue());
  if (Step <> 123) or (Written <> 23) then panic('write order'); end if;
  Step := 0;
  if InnerRead(OuterRead(O)) <> 17 then panic('read value'); end if;
  if Step <> 14 then panic('read order'); end if;
end program;"#,
    );
}

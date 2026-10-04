use super::*;

#[test]
fn record_methods_properties_and_events_execute() {
    assert_succeeds(
        r#"program RegisterMembers;

mutable var LastValue: integer := 0;
mutable var Handler: Option of (procedure(Value: integer)) := Option.None;

type Counter = record
  Value: integer;

  function Double(Self: Counter): integer;
  begin
    return Self.Value * 2;
  end function;

  function ReadNumber(Self: Counter): integer;
  begin
    return Self.Value;
  end function;

  procedure WriteNumber(Self: Counter; Value: integer);
  begin
    LastValue := Value;
  end procedure;

  property Number: integer read ReadNumber write WriteNumber;
end record;

type Button = record
  function ReadOnValue(Self: Button): Option of (procedure(Value: integer));
  begin
    return Handler;
  end function;

  procedure WriteOnValue(Self: Button; Value: Option of (procedure(Value: integer)));
  begin
    Handler := Value;
  end procedure;

  event OnValue: procedure(Value: integer) read ReadOnValue write WriteOnValue;
end record;

procedure Remember(Value: integer);
begin
  LastValue := Value;
end procedure;

begin
  var C: Counter := Counter(Value := 6);
  if C.Double() <> 12 then
    panic('method mismatch');
  end if;

  if C.Number <> 6 then
    panic('property read mismatch');
  end if;

  C.Number := 9;
  if LastValue <> 9 then
    panic('property write mismatch');
  end if;
  var B: Button := Button();
  if Assigned(B.OnValue) then
    panic('unexpected handler');
  end if;

  B.OnValue := Remember;
  if not Assigned(B.OnValue) then
    panic('missing handler');
  end if;

  B.OnValue(17);
  if LastValue <> 17 then
    panic('event raise mismatch');
  end if;

  B.OnValue := (nil);
  if Assigned(B.OnValue) then
    panic('handler was not cleared');
  end if;
end program;
"#,
    );
}

#[test]
fn readable_record_properties_keep_exact_getter_metadata() {
    let program = parse_ok(
        r#"program RecordPropertyMetadata;

type Counter = record
  Value: integer;

  function ReadNumber(Self: Counter): integer;
  begin
    return Self.Value;
  end function;

  property Number: integer read ReadNumber;
end record;

begin
  var C: Counter := Counter(Value := 1);
  if C.Number <> 1 then
    panic('property metadata fixture');
  end if;
end program;
"#,
    );
    let executable = crate::compile(&program).expect("property metadata source should compile");
    let executable = executable.executable();
    let record = executable
        .records
        .iter()
        .find(|record| executable.strings.get(record.name) == Some("Counter"))
        .expect("Counter record layout");
    let property = record
        .properties
        .first()
        .expect("readable property metadata");
    assert_eq!(executable.strings.get(property.name), Some("Number"));
    assert_eq!(
        executable.strings.get(property.getter),
        Some("Counter.ReadNumber")
    );
}

#[test]
fn static_and_generic_record_methods_use_resolved_targets() {
    assert_succeeds(
        r#"program RegisterGenericMethods;

type Box = record
  Value: integer;

  static function Create(Value: integer): Box;
  begin
    return Box(Value := Value);
  end function;

  function ReadNumber(Self: Box): integer;
  begin
    return Self.Value;
  end function;

  function Map of (T)(Self: Box; Transform: function(Value: integer): T): T;
  begin
    return Transform(Self.Value);
  end function;

  property Number: integer read ReadNumber;
end record;

function Double(Value: integer): integer;
begin
  return Value * 2;
end function;

begin
  var B: Box := box.create(11);
  if B.Map(Double) <> 22 then
    panic('generic method mismatch');
  end if;

  if Box.Create(7).Number <> 7 then
    panic('postfix property mismatch');
  end if;
end program;
"#,
    );
}

#[test]
fn bound_record_method_values_capture_the_receiver() {
    assert_succeeds(
        r#"program RegisterBoundMethod;

type Counter = record
  Base: integer;

  function Add(Self: Counter; Value: integer): integer;
  begin
    return Self.Base + Value;
  end function;
end record;

begin
  var C: Counter := Counter(Base := 10);
  var AddToCounter: function(Value: integer): integer := C.Add;
  if AddToCounter(7) <> 17 then
    panic('bound method mismatch');
  end if;
end program;
"#,
    );
}

#[test]
fn event_handlers_accept_bound_record_methods() {
    assert_succeeds(
        r#"program RegisterBoundEvent;

mutable var Handler: Option of (function(Value: integer): integer) := Option.None;

type Counter = record
  Base: integer;

  function Add(Self: Counter; Value: integer): integer;
  begin
    return Self.Base + Value;
  end function;
end record;

type Source = record
  function ReadValue(Self: Source): Option of (function(Value: integer): integer);
  begin
    return Handler;
  end function;

  procedure WriteValue(Self: Source; Value: Option of (function(Value: integer): integer));
  begin
    Handler := Value;
  end procedure;

  event OnValue: function(Value: integer): integer read ReadValue write WriteValue;
end record;

begin
  var C: Counter := Counter(Base := 12);
  var S: Source := Source();
  S.OnValue := C.Add;
  if S.OnValue(8) <> 20 then
    panic('bound event mismatch');
  end if;
end program;
"#,
    );
}

#[test]
fn chained_properties_evaluate_receiver_then_value_once() {
    assert_succeeds(
        r#"program RegisterPropertyOrder;

mutable var Step: integer := 0;
mutable var Written: integer := 0;

type Inner = record
  Value: integer;

  function ReadNumber(Self: Inner): integer;
  begin
    Step := Step * 10 + 4;
    return Self.Value;
  end function;

  procedure WriteNumber(Self: Inner; Value: integer);
  begin
    Step := Step * 10 + 3;
    Written := Value;
  end procedure;

  property Number: integer read ReadNumber write WriteNumber;
end record;

type Outer = record
  Item: Inner;

  function ReadChild(Self: Outer): Inner;
  begin
    Step := Step * 10 + 1;
    return Self.Item;
  end function;

  property Child: Inner read ReadChild;
end record;

function BuildValue(): integer;
begin
  Step := Step * 10 + 2;
  return 23;
end function;

begin
  var O: Outer := Outer(Item := Inner(Value := 17));
  O.Child.Number := BuildValue();
  if (Step <> 123) or (Written <> 23) then
    panic('property write order mismatch');
  end if;

  Step := 0;
  if O.Child.Number <> 17 then
    panic('property read mismatch');
  end if;

  if Step <> 14 then
    panic('property read order mismatch');
  end if;
end program;
"#,
    );
}

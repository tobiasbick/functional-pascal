use super::*;

#[test]
fn contextual_record_literals_expand_defaults_in_all_lowering_positions() {
    assert_succeeds(
        r#"
program ContextualRecords;
type Point = record
  X: integer := 0;
  Y: integer := 0;
end;
const OriginPoint: Point := record X := 0; end;
function Origin(): Point;
begin
  return record X := 4; end
end;
procedure Draw(P: Point);
begin
  if (P.X <> 0) or (P.Y <> 2) then panic('argument defaults')
end;
begin
  mutable var P: Point := record end;
  P := record X := 1; end;
  Draw(record Y := 2; end);
  var Points: array of Point := [record X := 3; end];
  var Returned: Point := Origin();
  if (P.X <> 1) or (P.Y <> 0) or
     (Points[0].X <> 3) or (Points[0].Y <> 0) or
     (Returned.X <> 4) or (Returned.Y <> 0) or
     (OriginPoint.X <> 0) or (OriginPoint.Y <> 0) then
    panic('contextual record defaults')
end.
"#,
    );
}

#[test]
fn globals_arrays_and_dictionaries_execute() {
    assert_succeeds(
        "\
program RegisterCollections;
mutable var Total: integer := 1;
begin
  mutable var Values: array of integer := [2, 3, 4];
  Values[1] := 8;
  mutable var Lookup: dict of string to integer := ['a': 5];
  Lookup['b'] := 7;
  Total := Total + Values[1] + Lookup['b'];
  if (Total <> 16) or not (8 in Values) or not ('b' in Lookup) then
    panic('collection mismatch')
end.",
    );
}

#[test]
fn records_defaults_updates_and_nested_cow_execute_on_register_path() {
    run_program(
        "\
program RegisterRecords;
type
  Point = record
    X: integer;
    Y: integer := 2;
  end;
begin
  var Original: Point := record X := 1; end;
  var Updated: Point := Original with X := 9; end;
  mutable var Items: array of Point := [Original, Updated];
  Items[0].Y := 7;
  if (Original.X <> 1) or (Original.Y <> 2) or (Updated.X <> 9) then
    panic('record copy mismatch');
  if (Items[0].Y <> 7) or (Items[1].Y <> 2) then
    panic('nested record mismatch')
end.",
    )
    .expect("register record path must succeed");
}

#[test]
fn result_option_and_data_enum_construction_execute() {
    assert_succeeds(
        "\
program RegisterVariants;
type
  Choice = enum
    Number(Value: integer);
    Empty;
  end;
begin
  var A: Result of integer, string := Ok(5);
  var B: Option of integer := Some(6);
  var C: Option of integer := None;
  var D: Choice := Choice.Number(7);
  if (A <> Ok(5)) or (B <> Some(6)) or (C <> None) then
    panic('variant mismatch')
end.",
    );
}

#[test]
fn aggregate_identifiers_remain_case_insensitive() {
    run_program(
        "\
program RegisterAggregateCase;
type
  Pair = record
    Left: integer;
    Right: integer;
  end;
begin
  mutable var VALUE: Pair := record Left := 1; Right := 2; end;
  value.lEfT := VALUE.right;
  if Value.Left <> 2 then panic('case mismatch')
end.",
    )
    .expect("register record names must be case-insensitive");
}

#[test]
fn try_unwraps_and_returns_early_for_result_and_option() {
    assert_succeeds(
        "\
program RegisterTry;
function ResultValue(Input: Result of integer, string): Result of integer, string;
begin
  var Value: integer := try Input;
  return Ok(Value + 1)
end;
function OptionValue(Input: Option of integer): Option of integer;
begin
  var Value: integer := try Input;
  return Some(Value + 1)
end;
begin
  if ResultValue(Ok(4)) <> Ok(5) then panic('result success');
  if ResultValue(Error('bad')) <> Error('bad') then panic('result failure');
  if OptionValue(Some(4)) <> Some(5) then panic('option success');
  if OptionValue(None) <> None then panic('option failure')
end.",
    );
}

#[test]
fn record_fields_survive_try_control_flow() {
    assert_succeeds(
        "\
program RegisterRecordTry;
type
  Triple = record
    First: integer;
    Second: integer;
    Third: integer;
  end;
function Read(Value: Result of integer, string): Result of integer, string;
begin
  return Value
end;
function Build(Second: Result of integer, string): Result of Triple, string;
begin
  return Ok(record
    First := 1;
    Second := try Read(Second);
    Third := try Read(Ok(3));
  end)
end;
begin
  case Build(Ok(2)) of
    Ok(Value):
      if (Value.First <> 1) or (Value.Second <> 2) or (Value.Third <> 3) then
        panic('record values');
    Error(Message): panic('unexpected record error')
  end;
  if Build(Error('expected')) <> Error('expected') then
    panic('record try propagation')
end.",
    );
}

#[test]
fn result_option_and_enum_patterns_bind_positional_fields() {
    assert_succeeds(
        "\
program RegisterPatterns;
type
  Shape = enum
    Point;
    Pair(Left: integer; Right: integer);
  end;
begin
  mutable var Sum: integer := 0;
  var ResultValue: Result of integer, string := Ok(3);
  case ResultValue of
    Ok(Value): Sum := Sum + Value;
    Error(Message): Sum := 99
  end;
  var OptionValue: Option of integer := Some(4);
  case OptionValue of
    Some(Value): Sum := Sum + Value;
    None: Sum := 99
  end;
  var ShapeValue: Shape := Shape.Pair(5, 6);
  case ShapeValue of
    Shape.Point: Sum := 99;
    Shape.Pair(A, B): Sum := Sum + A + B
  end;
  if Sum <> 18 then panic('pattern mismatch')
end.",
    );
}

#[test]
fn simple_enum_values_keep_backing_numbers_and_case_insensitivity() {
    assert_succeeds(
        "\
program RegisterSimpleEnum;
type
  State = enum
    Ready = 4;
    Running;
    Done = 9;
  end;
begin
  var Value: State := state.rUnNiNg;
  mutable var Number: integer := 0;
  case Value of
    State.Ready: Number := 4;
    State.Running: Number := 5;
    State.Done: Number := 9
  end;
  if Number <> 5 then panic('simple enum mismatch')
end.",
    );
}

#[test]
fn record_methods_properties_and_events_execute() {
    assert_succeeds(
        "\
program RegisterMembers;
mutable var LastValue: integer := 0;
mutable var Handler: Option of procedure(Value: integer) := None;

type
  Counter = record
    Value: integer;
    function Double(Self: Counter): integer;
    begin
      return Self.Value * 2
    end;
    function ReadNumber(Self: Counter): integer;
    begin
      return Self.Value
    end;
    procedure WriteNumber(Self: Counter; Value: integer);
    begin
      LastValue := Value
    end;
    property Number: integer read ReadNumber write WriteNumber;
  end;

  Button = record
    function ReadOnValue(Self: Button): Option of procedure(Value: integer);
    begin
      return Handler
    end;
    procedure WriteOnValue(Self: Button; Value: Option of procedure(Value: integer));
    begin
      Handler := Value
    end;
    event OnValue: procedure(Value: integer) read ReadOnValue write WriteOnValue;
  end;

procedure Remember(Value: integer);
begin
  LastValue := Value
end;

begin
  var C: Counter := record Value := 6; end;
  if C.Double() <> 12 then panic('method mismatch');
  if C.Number <> 6 then panic('property read mismatch');
  C.Number := 9;
  if LastValue <> 9 then panic('property write mismatch');

  var B: Button := record end;
  if Assigned(B.OnValue) then panic('unexpected handler');
  B.OnValue := Remember;
  if not Assigned(B.OnValue) then panic('missing handler');
  B.OnValue(17);
  if LastValue <> 17 then panic('event raise mismatch');
  B.OnValue := (nil);
  if Assigned(B.OnValue) then panic('handler was not cleared')
end.",
    );
}

#[test]
fn readable_record_properties_keep_exact_getter_metadata() {
    let program = parse_ok(
        "\
program RecordPropertyMetadata;
type
  Counter = record
    Value: integer;
    function ReadNumber(Self: Counter): integer;
    begin
      return Self.Value
    end;
    property Number: integer read ReadNumber;
  end;
begin
  var C: Counter := record Value := 1; end;
  if C.Number <> 1 then panic('property metadata fixture')
end.",
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
fn string_indexing_and_membership_execute() {
    assert_succeeds(
        "\
program RegisterStringAggregateOps;
begin
  var Text: string := 'Hällo';
  if Text[1] <> 'ä' then panic('unicode string index mismatch');
  if not ('äll' in Text) then panic('substring membership mismatch');
  if not ('ä' in Text) then panic('character membership mismatch')
end.",
    );
}

#[test]
fn anonymous_record_shapes_use_positional_fields() {
    assert_succeeds(
        "\
program RegisterAnonymousRecord;
begin
  if (record Left := 3; Right := 4; end).Left <> 3 then
    panic('anonymous record mismatch')
end.",
    );
}

#[test]
fn generic_routines_preserve_record_and_enum_values() {
    assert_succeeds(
        "\
program RegisterGenericAggregates;
type
  Point = record
    X: integer;
  end;
  Choice = enum
    Number(Value: integer);
    Empty;
  end;
function Identity<T>(Value: T): T;
begin
  return Value
end;
begin
  var P: Point := Identity(record X := 8; end);
  if P.X <> 8 then panic('generic record mismatch');
  var C: Choice := Identity(Choice.Number(9));
  case C of
    Choice.Number(Value): if Value <> 9 then panic('generic enum payload mismatch');
    Choice.Empty: panic('generic enum variant mismatch')
  end
end.",
    );
}

#[test]
fn static_and_generic_record_methods_use_resolved_targets() {
    assert_succeeds(
        "\
program RegisterGenericMethods;
type
  Box = record
    Value: integer;
    static function Create(Value: integer): Box;
    begin
      return record Value := Value; end
    end;
    function ReadNumber(Self: Box): integer;
    begin
      return Self.Value
    end;
    property Number: integer read ReadNumber;
    function Map<T>(Self: Box; Transform: function(Value: integer): T): T;
    begin
      return Transform(Self.Value)
    end;
  end;
function Double(Value: integer): integer;
begin
  return Value * 2
end;
begin
  var B: Box := box.create(11);
  if B.Map(Double) <> 22 then panic('generic method mismatch');
  if Box.Create(7).Number <> 7 then panic('postfix property mismatch')
end.",
    );
}

#[test]
fn bound_record_method_values_capture_the_receiver() {
    assert_succeeds(
        "\
program RegisterBoundMethod;
type
  Counter = record
    Base: integer;
    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
  end;
begin
  var C: Counter := record Base := 10; end;
  var AddToCounter: function(Value: integer): integer := C.Add;
  if AddToCounter(7) <> 17 then panic('bound method mismatch')
end.",
    );
}

#[test]
fn event_handlers_accept_bound_record_methods() {
    assert_succeeds(
        "\
program RegisterBoundEvent;
mutable var Handler: Option of function(Value: integer): integer := None;
type
  Counter = record
    Base: integer;
    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
  end;
  Source = record
    function ReadValue(Self: Source): Option of function(Value: integer): integer;
    begin
      return Handler
    end;
    procedure WriteValue(
      Self: Source;
      Value: Option of function(Value: integer): integer
    );
    begin
      Handler := Value
    end;
    event OnValue: function(Value: integer): integer read ReadValue write WriteValue;
  end;
begin
  var C: Counter := record Base := 12; end;
  var S: Source := record end;
  S.OnValue := C.Add;
  if S.OnValue(8) <> 20 then panic('bound event mismatch')
end.",
    );
}

#[test]
fn chained_properties_evaluate_receiver_then_value_once() {
    assert_succeeds(
        "\
program RegisterPropertyOrder;
mutable var Step: integer := 0;
mutable var Written: integer := 0;
type
  Inner = record
    Value: integer;
    function ReadNumber(Self: Inner): integer;
    begin
      Step := Step * 10 + 4;
      return Self.Value
    end;
    procedure WriteNumber(Self: Inner; Value: integer);
    begin
      Step := Step * 10 + 3;
      Written := Value
    end;
    property Number: integer read ReadNumber write WriteNumber;
  end;
  Outer = record
    Item: Inner;
    function ReadChild(Self: Outer): Inner;
    begin
      Step := Step * 10 + 1;
      return Self.Item
    end;
    property Child: Inner read ReadChild;
  end;
function BuildValue(): integer;
begin
  Step := Step * 10 + 2;
  return 23
end;
begin
  var O: Outer := record Item := record Value := 17; end; end;
  O.Child.Number := BuildValue();
  if (Step <> 123) or (Written <> 23) then panic('property write order mismatch');
  Step := 0;
  if O.Child.Number <> 17 then panic('property read mismatch');
  if Step <> 14 then panic('property read order mismatch')
end.",
    );
}

#[test]
fn array_push_uses_direct_opcode_and_preserves_value_aliases() {
    let source = "\
program RegisterArrayPush;
uses Std.Array;
begin
  mutable var A: array of integer := [1];
  var Original: array of integer := A;
  Push(A, 2);
  if Length(Original) <> 1 then panic('array alias changed');
  if Length(A) <> 2 then panic('array push length mismatch');
  if A[1] <> 2 then panic('array push value mismatch')
end.";
    assert_succeeds(source);

    let program = super::parse_ok(source);
    let executable = crate::compile(&program).expect("compilation should succeed");
    assert!(
        executable
            .executable()
            .code
            .iter()
            .any(|instruction| { instruction.opcode() == Ok(fpas_bytecode::Opcode::ArrayPush) })
    );
}

#[test]
fn global_nested_index_write_uses_direct_path_and_preserves_value_aliases() {
    let source = "\
program RegisterGlobalIndexPath;
mutable var Surface: array of array of integer := [[1, 2]];
begin
  var Original: array of array of integer := Surface;
  Surface[0][1] := 9;
  if Original[0][1] <> 2 then panic('global alias changed');
  if Surface[0][1] <> 9 then panic('global path value mismatch')
end.";
    assert_succeeds(source);

    let program = super::parse_ok(source);
    let executable = crate::compile(&program).expect("compilation should succeed");
    assert!(executable.executable().code.iter().any(|instruction| {
        instruction.opcode() == Ok(fpas_bytecode::Opcode::StoreGlobalIndexPath)
    }));
}

#[test]
fn global_nested_index_write_preserves_index_side_effect_order() {
    assert_succeeds(
        "\
program RegisterGlobalIndexOrder;
mutable var Surface: array of array of integer := [[1, 2]];
function ChangeSurface(): integer;
begin
  Surface := [[3, 4]];
  return 1
end;
begin
  Surface[0][ChangeSurface()] := 9;
  if Surface[0][0] <> 1 then panic('snapshot order changed');
  if Surface[0][1] <> 9 then panic('snapshot update missing')
end.",
    );
}

#[test]
fn global_nested_dictionary_write_inserts_leaf_and_preserves_aliases() {
    assert_succeeds(
        "\
program RegisterGlobalDictionaryPath;
mutable var Lookup: dict of string to dict of string to integer := ['outer': ['old': 1]];
begin
  var Original: dict of string to dict of string to integer := Lookup;
  Lookup['outer']['new'] := 2;
  if 'new' in Original['outer'] then panic('dictionary alias changed');
  if Lookup['outer']['new'] <> 2 then panic('dictionary path value mismatch')
end.",
    );
}

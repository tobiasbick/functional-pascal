use super::*;

mod record_updates;
mod structural_equality;
mod try_expressions;
mod type_order;

#[test]
fn contextual_record_literals_expand_defaults_in_all_lowering_positions() {
    assert_succeeds(
        r#"
program ContextualRecords;
type Point = record
  X: integer := 0;
  Y: integer := 0;
end record;
const OriginPoint: Point := record X := 0; end;
function Origin(): Point;
begin
  return record X := 4; end;
end function;
procedure Draw(P: Point);
begin
  if (P.X <> 0) or (P.Y <> 2) then panic('argument defaults'); end if;
end procedure;
begin
  var P: Point := record end;
  P := record X := 1; end;
  Draw(record Y := 2; end);
  const Points: array of Point := [record X := 3; end];
  const Returned: Point := Origin();
  if (P.X <> 1) or (P.Y <> 0) or
     (Points[0].X <> 3) or (Points[0].Y <> 0) or
     (Returned.X <> 4) or (Returned.Y <> 0) or
     (OriginPoint.X <> 0) or (OriginPoint.Y <> 0) then
    panic('contextual record defaults'); end if;
end.
"#,
    );
}

#[test]
fn globals_arrays_and_dictionaries_execute() {
    assert_succeeds(
        "\
program RegisterCollections;
var Total: integer := 1;
begin
  var Values: array of integer := [2, 3, 4];
  Values[1] := 8;
  var Lookup: dict of string to integer := ['a': 5];
  Lookup['b'] := 7;
  Total := Total + Values[1] + Lookup['b'];
  if (Total <> 16) or not (8 in Values) or not ('b' in Lookup) then
    panic('collection mismatch'); end if;
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
  end record;
begin
  const Original: Point := record X := 1; end;
  const Updated: Point := Original with X := 9; end with;
  var Items: array of Point := [Original, Updated];
  Items[0].Y := 7;
  if (Original.X <> 1) or (Original.Y <> 2) or (Updated.X <> 9) then
    panic('record copy mismatch'); end if;
  if (Items[0].Y <> 7) or (Items[1].Y <> 2) then
    panic('nested record mismatch'); end if;
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
  end enum;
begin
  const A: Result of integer, string := Ok(5);
  const B: Option of integer := Some(6);
  const C: Option of integer := None;
  const D: Choice := Choice.Number(7);
  if (A <> Ok(5)) or (B <> Some(6)) or (C <> None) then
    panic('variant mismatch'); end if;
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
  end record;
begin
  var VALUE: Pair := record Left := 1; Right := 2; end;
  value.lEfT := VALUE.right;
  if Value.Left <> 2 then panic('case mismatch'); end if;
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
  const Value: integer := try Input;
  return Ok(Value + 1);
end function;
function OptionValue(Input: Option of integer): Option of integer;
begin
  const Value: integer := try Input;
  return Some(Value + 1);
end function;
begin
  if ResultValue(Ok(4)) <> Ok(5) then panic('result success'); end if;
  if ResultValue(Error('bad')) <> Error('bad') then panic('result failure'); end if;
  if OptionValue(Some(4)) <> Some(5) then panic('option success'); end if;
  if OptionValue(None) <> None then panic('option failure'); end if;
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
  end record;
function ReadValue(Value: Result of integer, string): Result of integer, string;
begin
  return Value;
end function;
function Build(Second: Result of integer, string): Result of Triple, string;
begin
  return Ok(record
    First := 1;
    Second := try ReadValue(Second);
    Third := try ReadValue(Ok(3));
  end);
end function;
begin
  case Build(Ok(2)) of
    when Ok(Value):
      if (Value.First <> 1) or (Value.Second <> 2) or (Value.Third <> 3) then
        panic('record values'); end if;
    when Error(Message): panic('unexpected record error');
  end case;
  if Build(Error('expected')) <> Error('expected') then
    panic('record try propagation'); end if;
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
  end enum;
begin
  var Sum: integer := 0;
  const ResultValue: Result of integer, string := Ok(3);
  case ResultValue of
    when Ok(Value): Sum := Sum + Value;
    when Error(Message): Sum := 99;
  end case;
  const OptionValue: Option of integer := Some(4);
  case OptionValue of
    when Some(Value): Sum := Sum + Value;
    when None: Sum := 99;
  end case;
  const ShapeValue: Shape := Shape.Pair(5, 6);
  case ShapeValue of
    when Shape.Point: Sum := 99;
    when Shape.Pair(A, B): Sum := Sum + A + B;
  end case;
  if Sum <> 18 then panic('pattern mismatch'); end if;
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
  end enum;
  type StateAlias = State;
begin
  const Value: State := state.rUnNiNg;
  const AliasValue: StateAlias := StateAlias.Done;
  var Number: integer := 0;
  case Value of
    when State.Ready: Number := 4;
    when State.Running: Number := 5;
    when State.Done: Number := 9;
  end case;
  if Number <> 5 then panic('simple enum mismatch'); end if;
  case StateAlias.Running of
    when State.Ready: Number := 99;
    when State.Running: Number := Number + 1;
    when State.Done: Number := 99;
  end case;
  case AliasValue of
    when State.Ready: Number := 99;
    when State.Running: Number := 99;
    when State.Done: Number := Number + 1;
  end case;
  if Number <> 7 then panic('alias enum backing mismatch'); end if;
end.",
    );
}

#[test]
fn record_methods_properties_and_events_execute() {
    assert_succeeds(
        "\
program RegisterMembers;
var LastValue: integer := 0;
var Handler: Option of procedure(Value: integer) := None;

type
  Counter = record
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
    function ReadOnValue(Self: Button): Option of procedure(Value: integer);
    begin
      return Handler;
    end function;
    procedure WriteOnValue(Self: Button; Value: Option of procedure(Value: integer));
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
  const C: Counter := record Value := 6; end;
  if C.Double() <> 12 then panic('method mismatch'); end if;
  if C.Number <> 6 then panic('property read mismatch'); end if;
  C.Number := 9;
  if LastValue <> 9 then panic('property write mismatch'); end if;

  const B: Button := record end;
  if Assigned(B.OnValue) then panic('unexpected handler'); end if;
  B.OnValue := Remember;
  if not Assigned(B.OnValue) then panic('missing handler'); end if;
  B.OnValue(17);
  if LastValue <> 17 then panic('event raise mismatch'); end if;
  B.OnValue := (nil);
  if Assigned(B.OnValue) then panic('handler was not cleared'); end if;
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
      return Self.Value;
    end function;
    property Number: integer read ReadNumber;
  end record;
begin
  const C: Counter := record Value := 1; end;
  if C.Number <> 1 then panic('property metadata fixture'); end if;
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
  const Text: string := 'Hällo';
  if Text[1] <> 'ä' then panic('unicode string index mismatch'); end if;
  if not ('äll' in Text) then panic('substring membership mismatch'); end if;
  if not ('ä' in Text) then panic('character membership mismatch'); end if;
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
    panic('anonymous record mismatch'); end if;
end.",
    );
}

#[test]
fn inferred_and_contextual_records_preserve_initializer_order() {
    assert_succeeds(
        r#"
program RecordInitializerOrder;
type Pair = record
  First: integer;
  Second: integer := 7;
end record;
var Calls: integer := 0;
function Next(): integer;
begin
  Calls := Calls + 1;
  return Calls;
end function;
begin
  if (record First := Next(); Second := Next(); end).Second <> 2 then
    panic('anonymous record initializer order'); end if;
  const Typed: Pair := record First := Next(); end;
  if (Typed.First <> 3) or (Typed.Second <> 7) or (Calls <> 3) then
    panic('record initializer order'); end if;
end.
"#,
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
  end record;
  type Choice = enum
    Number(Value: integer);
    Empty;
  end enum;
function Identity<T>(Value: T): T;
begin
  return Value;
end function;
begin
  const P: Point := Identity(record X := 8; end);
  if P.X <> 8 then panic('generic record mismatch'); end if;
  const C: Choice := Identity(Choice.Number(9));
  case C of
    when Choice.Number(Value): if Value <> 9 then panic('generic enum payload mismatch'); end if;
    when Choice.Empty: panic('generic enum variant mismatch');
  end case;
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
      return record Value := Value; end;
    end function;
    function ReadNumber(Self: Box): integer;
    begin
      return Self.Value;
    end function;
    property Number: integer read ReadNumber;
    function Map<T>(Self: Box; Transform: function(Value: integer): T): T;
    begin
      return Transform(Self.Value);
    end function;
  end record;
function Double(Value: integer): integer;
begin
  return Value * 2;
end function;
begin
  const B: Box := box.create(11);
  if B.Map(Double) <> 22 then panic('generic method mismatch'); end if;
  if Box.Create(7).Number <> 7 then panic('postfix property mismatch'); end if;
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
      return Self.Base + Value;
    end function;
  end record;
begin
  const C: Counter := record Base := 10; end;
  const AddToCounter: function(Value: integer): integer := C.Add;
  if AddToCounter(7) <> 17 then panic('bound method mismatch'); end if;
end.",
    );
}

#[test]
fn event_handlers_accept_bound_record_methods() {
    assert_succeeds(
        "\
program RegisterBoundEvent;
var Handler: Option of function(Value: integer): integer := None;
type
  Counter = record
    Base: integer;
    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value;
    end function;
  end record;
  type Source = record
    function ReadValue(Self: Source): Option of function(Value: integer): integer;
    begin
      return Handler;
    end function;
    procedure WriteValue(
      Self: Source;
      Value: Option of function(Value: integer): integer
    );
    begin
      Handler := Value;
    end procedure;
    event OnValue: function(Value: integer): integer read ReadValue write WriteValue;
  end record;
begin
  const C: Counter := record Base := 12; end;
  const S: Source := record end;
  S.OnValue := C.Add;
  if S.OnValue(8) <> 20 then panic('bound event mismatch'); end if;
end.",
    );
}

#[test]
fn chained_properties_evaluate_receiver_then_value_once() {
    assert_succeeds(
        "\
program RegisterPropertyOrder;
var Step: integer := 0;
var Written: integer := 0;
type
  Inner = record
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
  const O: Outer := record Item := record Value := 17; end; end;
  O.Child.Number := BuildValue();
  if (Step <> 123) or (Written <> 23) then panic('property write order mismatch'); end if;
  Step := 0;
  if O.Child.Number <> 17 then panic('property read mismatch'); end if;
  if Step <> 14 then panic('property read order mismatch'); end if;
end.",
    );
}

mod arrays;

#[test]
fn global_nested_index_write_uses_direct_path_and_preserves_value_aliases() {
    let source = "\
program RegisterGlobalIndexPath;
var Surface: array of array of integer := [[1, 2]];
begin
  const Original: array of array of integer := Surface;
  Surface[0][1] := 9;
  if Original[0][1] <> 2 then panic('global alias changed'); end if;
  if Surface[0][1] <> 9 then panic('global path value mismatch'); end if;
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
var Surface: array of array of integer := [[1, 2]];
function ChangeSurface(): integer;
begin
  Surface := [[3, 4]];
  return 1;
end function;
begin
  Surface[0][ChangeSurface()] := 9;
  if Surface[0][0] <> 1 then panic('snapshot order changed'); end if;
  if Surface[0][1] <> 9 then panic('snapshot update missing'); end if;
end.",
    );
}

#[test]
fn global_nested_dictionary_write_inserts_leaf_and_preserves_aliases() {
    assert_succeeds(
        "\
program RegisterGlobalDictionaryPath;
var Lookup: dict of string to dict of string to integer := ['outer': ['old': 1]];
begin
  const Original: dict of string to dict of string to integer := Lookup;
  Lookup['outer']['new'] := 2;
  if 'new' in Original['outer'] then panic('dictionary alias changed'); end if;
  if Lookup['outer']['new'] <> 2 then panic('dictionary path value mismatch'); end if;
end.",
    );
}

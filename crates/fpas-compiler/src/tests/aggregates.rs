use super::*;

mod generic_enums;
mod generic_records;
mod handler_fields;
mod mutating_arrays;
mod record_construction;
mod record_updates;
mod structural_equality;
mod try_expressions;
mod type_order;

#[test]
fn typed_record_constructions_expand_defaults_in_all_lowering_positions() {
    assert_succeeds(
        r#"
program ContextualRecords;
type Point = record
  X: integer := 0;
  Y: integer := 0;
end record;
const OriginPoint: Point := Point( X := 0 );
function Origin(): Point;
begin
  return Point( X := 4 );
end function;
procedure Draw(P: Point);
begin
  if (P.X <> 0) or (P.Y <> 2) then panic('argument defaults'); end if;
end procedure;
begin
  var P: Point := Point( );
  P := Point( X := 1 );
  Draw(Point( Y := 2 ));
  const Points: array of Point := [Point( X := 3 )];
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
  const Original: Point := Point( X := 1 );
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
  const A: Result of (integer, string) := Ok(5);
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
  var VALUE: Pair := Pair( Left := 1, Right := 2 );
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
function ResultValue(Input: Result of (integer, string)): Result of (integer, string);
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
function ReadValue(Value: Result of (integer, string)): Result of (integer, string);
begin
  return Value;
end function;
function Build(Second: Result of (integer, string)): Result of (Triple, string);
begin
  return Ok(Triple(
    First := 1,
    Second := try ReadValue(Second),
    Third := try ReadValue(Ok(3))
  ));
end function;
begin
  case Build(Ok(2)) of
    when Ok(const Value):
      if (Value.First <> 1) or (Value.Second <> 2) or (Value.Third <> 3) then
        panic('record values'); end if;
    when Error(const Message): panic('unexpected record error');
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
  const ResultValue: Result of (integer, string) := Ok(3);
  case ResultValue of
    when Ok(const Value): Sum := Sum + Value;
    when Error(const Message): Sum := 99;
  end case;
  const OptionValue: Option of integer := Some(4);
  case OptionValue of
    when Some(const Value): Sum := Sum + Value;
    when None: Sum := 99;
  end case;
  const ShapeValue: Shape := Shape.Pair(5, 6);
  case ShapeValue of
    when Shape.Point: Sum := 99;
    when Shape.Pair(const A, const B): Sum := Sum + A + B;
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
fn typed_record_constructions_expose_named_fields() {
    assert_succeeds(
        "\
program RegisterConstructedRecord;
type Pair = record Left: integer; Right: integer; end record;
begin
  if Pair(Left := 3, Right := 4).Left <> 3 then
    panic('constructed record mismatch'); end if;
end.",
    );
}

#[test]
fn typed_records_preserve_migrated_initializer_order() {
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
  if (Pair( First := Next(), Second := Next() )).Second <> 2 then
    panic('record initializer order'); end if;
  const Typed: Pair := Pair( First := Next() );
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
  const P: Point := Identity(Point( X := 8 ));
  if P.X <> 8 then panic('generic record mismatch'); end if;
  const C: Choice := Identity(Choice.Number(9));
  case C of
    when Choice.Number(const Value): if Value <> 9 then panic('generic enum payload mismatch'); end if;
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
      return Box( Value := Value );
    end function;
    function ReadNumber(Self: Box): integer;
    begin
      return Self.Value;
    end function;
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
  if Box.Create(7).ReadNumber() <> 7 then panic('postfix method mismatch'); end if;
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
  const C: Counter := Counter( Base := 10 );
  const AddToCounter: function(Value: integer): integer := C.Add;
  if AddToCounter(7) <> 17 then panic('bound method mismatch'); end if;
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

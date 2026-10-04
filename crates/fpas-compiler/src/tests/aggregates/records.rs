use super::*;

#[test]
fn record_construction_expands_defaults_in_all_lowering_positions() {
    assert_succeeds(
        r#"program ContextualRecords;

type Point = record
  X: integer := 0;
  Y: integer := 0;
end record;

const OriginPoint: Point := Point(X := 0);

function Origin(): Point;
begin
  return Point(X := 4);
end function;

procedure Draw(P: Point);
begin
  if (P.X <> 0) or (P.Y <> 2) then
    panic('argument defaults');
  end if;
end procedure;

begin
  mutable var P: Point := Point();
  P := Point(X := 1);
  Draw(Point(Y := 2));
  var Points: array of (Point) := [Point(X := 3)];
  var Returned: Point := Origin();
  if (P.X <> 1) or (P.Y <> 0) or (Points[0].X <> 3) or (Points[0].Y <> 0) or (Returned.X <> 4) or (Returned.Y <> 0) or (OriginPoint.X <> 0) or
     (OriginPoint.Y <> 0) then
    panic('contextual record defaults');
  end if;
end program;
"#,
    );
}

#[test]
fn records_defaults_updates_and_nested_cow_execute_on_register_path() {
    run_program(
        r#"program RegisterRecords;

type Point = record
  X: integer;
  Y: integer := 2;
end record;

begin
  var Original: Point := Point(X := 1);
  var Updated: Point := Original with X := 9; end with;
  mutable var Items: array of (Point) := [Original, Updated];
  Items[0].Y := 7;
  if (Original.X <> 1) or (Original.Y <> 2) or (Updated.X <> 9) then
    panic('record copy mismatch');
  end if;

  if (Items[0].Y <> 7) or (Items[1].Y <> 2) then
    panic('nested record mismatch');
  end if;
end program;
"#,
    )
    .expect("register record path must succeed");
}

#[test]
fn record_fields_survive_try_control_flow() {
    assert_succeeds(
        r#"program RegisterRecordTry;

type Triple = record
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
  return Result.Ok(Triple(First := 1, Second := try ReadValue(Second), Third := try ReadValue(Result.Ok(3))));
end function;

begin
  case Build(Result.Ok(2)) of
    when Result.Ok(const Value):
      if (Value.First <> 1) or (Value.Second <> 2) or (Value.Third <> 3) then
        panic('record values');
      end if;
    when Result.Error(const Message):
      panic('unexpected record error');
  end case;

  if Build(Result.Error('expected')) <> Result.Error('expected') then
    panic('record try propagation');
  end if;
end program;
"#,
    );
}

#[test]
fn named_record_construction_supports_immediate_field_projection() {
    assert_succeeds(
        r#"program RegisterRecordProjection;
type Pair = record Left: integer; Right: integer; end record;
begin
  if Pair(Left := 3, Right := 4).Left <> 3 then
    panic('record projection mismatch'); end if;
end program;"#,
    );
}

#[test]
fn record_construction_preserves_initializer_order_and_defaults() {
    assert_succeeds(
        r#"
program RecordInitializerOrder;
 type Pair = record
  First: integer;
  Second: integer := 7;
end record;
  mutable var Calls: integer := 0;
function Next(): integer;
begin
  Calls := Calls + 1;
  return Calls;
end function;
begin
  if Pair(First := Next(), Second := Next()).Second <> 2 then
    panic('record initializer order'); end if;
  var Typed: Pair := Pair(First := Next());
  if (Typed.First <> 3) or (Typed.Second <> 7) or (Calls <> 3) then
    panic('record initializer order'); end if;
end program;
"#,
    );
}

#[test]
fn generic_routines_preserve_record_and_enum_values() {
    assert_succeeds(
        r#"program RegisterGenericAggregates;

  type Point = record
    X: integer;
  end record;
  type Choice = enum
    Number(Value: integer);
    Empty;
  end enum;
function Identity of (T)(Value: T): T;
begin
  return Value;
end function;
begin
  var P: Point := Identity(Point(X := 8));
  if P.X <> 8 then panic('generic record mismatch'); end if;
  var C: Choice := Identity(Choice.Number(9));
  case C of
    when Choice.Number(const Value): if Value <> 9 then panic('generic enum payload mismatch'); end if;
    when Choice.Empty: panic('generic enum variant mismatch');
  end case;
end program;"#,
    );
}

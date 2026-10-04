use super::*;

#[test]
fn finite_scalar_cases_return_values_without_an_implicit_unit_path() {
    assert_succeeds(
        "program Main;
         function Choose(Value: boolean): integer;
         begin case Value of when true: return 1; when false: return 2; end case;
         end function;
         function Identity(Value: integer): integer;
         begin case Value of when const Bound: return Bound; end case;
         end function;
         begin
           if Choose(true) <> 1 then panic('true arm'); end if;
           if Choose(false) <> 2 then panic('false arm'); end if;
           if Identity(42) <> 42 then panic('binding arm'); end if;
         end program;",
    );
}

#[test]
fn result_option_and_data_enum_construction_execute() {
    assert_succeeds(
        r#"program RegisterVariants;

  type Choice = enum
    Number(Value: integer);
    Empty;
  end enum;
begin
  const A: Result of (integer, string) := Result.Ok(5);
  const B: Option of (integer) := Option.Some(6);
  const C: Option of (integer) := Option.None;
  const D: Choice := Choice.Number(7);
  if (A <> Result.Ok(5)) or (B <> Option.Some(6)) or (C <> Option.None) then
    panic('variant mismatch'); end if;
end program;"#,
    );
}

#[test]
fn aggregate_identifiers_remain_case_insensitive() {
    run_program(
        r#"program RegisterAggregateCase;

type Pair = record
  Left: integer;
  Right: integer;
end record;

begin
   var VALUE: Pair := Pair(Left := 1, Right := 2);
  value.lEfT := VALUE.right;
  if Value.Left <> 2 then
    panic('case mismatch');
  end if;
end program;
"#,
    )
    .expect("register record names must be case-insensitive");
}

#[test]
fn try_unwraps_and_returns_early_for_result_and_option() {
    assert_succeeds(
        r#"program RegisterTry;
function ResultValue(Input: Result of (integer, string)): Result of (integer, string);
begin
  const Value: integer := try Input;
  return Result.Ok(Value + 1);
end function;
function OptionValue(Input: Option of (integer)): Option of (integer);
begin
  const Value: integer := try Input;
  return Option.Some(Value + 1);
end function;
begin
  if ResultValue(Result.Ok(4)) <> Result.Ok(5) then panic('result success'); end if;
  if ResultValue(Result.Error('bad')) <> Result.Error('bad') then panic('result failure'); end if;
  if OptionValue(Option.Some(4)) <> Option.Some(5) then panic('option success'); end if;
  if OptionValue(Option.None) <> Option.None then panic('option failure'); end if;
end program;"#,
    );
}

#[test]
fn result_option_and_enum_patterns_bind_positional_fields() {
    assert_succeeds(
        r#"program RegisterPatterns;

type Shape = enum
  Point;
  Pair(Left: integer; Right: integer);
end enum;

begin
   var Sum: integer := 0;
  const ResultValue: Result of (integer, string) := Result.Ok(3);
  case ResultValue of
    when Result.Ok(const Value):
      Sum := Sum + Value;
    when Result.Error(const Message):
      Sum := 99;
  end case;
  const OptionValue: Option of (integer) := Option.Some(4);
  case OptionValue of
    when Option.Some(const Value):
      Sum := Sum + Value;
    when Option.None:
      Sum := 99;
  end case;
  const ShapeValue: Shape := Shape.Pair(5, 6);
  case ShapeValue of
    when Shape.Point:
      Sum := 99;
    when Shape.Pair(const A, const B):
      Sum := Sum + A + B;
  end case;

  if Sum <> 18 then
    panic('pattern mismatch');
  end if;
end program;
"#,
    );
}

#[test]
fn simple_enum_values_keep_backing_numbers_and_case_insensitivity() {
    assert_succeeds(
        r#"program RegisterSimpleEnum;

  type State = enum
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
  const Selected: integer := case Value of
    when State.Ready: 4;
    when State.Running: 5;
    when State.Done: 9;
  end case;
  if Selected <> 5 then panic('enum expression backing mismatch'); end if;
end program;"#,
    );
}

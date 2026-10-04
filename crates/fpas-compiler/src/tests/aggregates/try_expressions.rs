use super::super::assert_succeeds;

fn check_expression(body: &str) {
    assert_succeeds(&format!(
        r#"
program TryExpressions;
uses Std.Tasks as Tasks;
   var Written: integer := 0;
   var Grid: array of (array of (integer)) := [[0, 0, 0], [0, 0, 0]];

  type Binary = function(X: integer; Y: integer): integer;
  type Bucket = record Items: array of (integer); end record;
  type Counter = record
    Base: integer;
    OnValue: Option of (Binary) := Option.None;
  end record;
  type Message = enum Move(X: integer; Y: integer); end enum;
  type Pair = record First: integer; Second: integer; end record;
   var Trace: integer := 0;
function CounterAdd(Receiver: Counter; X: integer; Y: integer): integer;
begin return Receiver.Base + X * 10 + Y; end function;
procedure CounterWriteNumber(Receiver: Counter; Value: integer);
begin Written := Receiver.Base + Value; end procedure;
function ReadValue(Value: integer; FailAt: integer): result of (integer, string);
begin
  Trace := Trace * 10 + Value;
  if Value = FailAt then return Result.Error('expected'); end if;
  return Result.Ok(Value);
end function;
function Combine(X: integer; Y: integer): integer;
begin
  return X * 10 + Y;
end function;
function ReadHandler(FailAt: integer): result of (Binary, string);
begin
  const X: integer := try ReadValue(1, FailAt);
  const Y: integer := try ReadValue(2, FailAt);
  return Result.Ok(Combine);
end function;
function Probe(FailAt: integer): result of (integer, string);
begin
  {body}
end function;
begin
  if Probe(0) <> Result.Ok(12) then panic('success value'); end if;
  if Trace <> 12 then panic('success evaluation order'); end if;
  Trace := 0;
  if Probe(1) <> Result.Error('expected') then panic('first error'); end if;
  if Trace <> 1 then panic('evaluated after first error'); end if;
  Trace := 0;
  if Probe(2) <> Result.Error('expected') then panic('second error'); end if;
  if Trace <> 12 then panic('second error evaluation order'); end if;
end program;
"#
    ));
}

#[test]
fn enum_constructor_preserves_earlier_try_arguments() {
    check_expression(
        "var Value: Message := Message.Move(try ReadValue(1, FailAt), try ReadValue(2, FailAt)); case Value of when Message.Move(const X, const Y): return Result.Ok(X * 10 + Y); end case;",
    );
}

#[test]
fn direct_call_preserves_earlier_try_arguments() {
    check_expression(
        "return Result.Ok(Combine(try ReadValue(1, FailAt), try ReadValue(2, FailAt)));",
    );
}

#[test]
fn array_literal_preserves_earlier_try_elements() {
    check_expression(
        "var Values: array of (integer) := [try ReadValue(1, FailAt), try ReadValue(2, FailAt)]; return Result.Ok(Values[0] * 10 + Values[1]);",
    );
}

#[test]
fn dictionary_literal_preserves_try_key_across_try_value() {
    check_expression(
        "var Values: dict of (integer, integer) := [try ReadValue(1, FailAt): try ReadValue(2, FailAt)]; return Result.Ok(10 + Values[1]);",
    );
}

#[test]
fn binary_expression_preserves_left_operand_across_try() {
    check_expression(
        "return Result.Ok((try ReadValue(1, FailAt)) * 10 + (try ReadValue(2, FailAt)));",
    );
}

#[test]
fn record_update_preserves_base_and_fields_across_try() {
    check_expression(
        "var Original: Pair := Pair(First := 8, Second := 9); var Value: Pair := Original with First := try ReadValue(1, FailAt); Second := try ReadValue(2, FailAt); end with; return Result.Ok(Value.First * 10 + Value.Second);",
    );
}

#[test]
fn function_value_preserves_callee_across_try() {
    check_expression(
        "var F: function(X: integer; Y: integer): integer := Combine; return Result.Ok(F(try ReadValue(1, FailAt), try ReadValue(2, FailAt)));",
    );
}

#[test]
fn task_spawn_preserves_callee_and_arguments_across_try() {
    check_expression(
        "return Result.Ok(Tasks.Wait(go Combine(try ReadValue(1, FailAt), try ReadValue(2, FailAt))));",
    );
}

#[test]
fn index_read_preserves_collection_across_try() {
    check_expression(
        "var Values: array of (integer) := [0, 10]; return Result.Ok(Values[try ReadValue(1, FailAt)] + (try ReadValue(2, FailAt)));",
    );
}

#[test]
fn nested_index_write_preserves_path_and_replacement_across_try() {
    check_expression(
        "var Values: array of (array of (integer)) := [[0, 0, 0], [0, 0, 0]]; Values[try ReadValue(1, FailAt)][try ReadValue(2, FailAt)] := 12; return Result.Ok(Values[1][2]);",
    );
}

#[test]
fn global_index_write_preserves_path_and_replacement_across_try() {
    check_expression(
        "Grid[try ReadValue(1, FailAt)][try ReadValue(2, FailAt)] := 12; return Result.Ok(Grid[1][2]);",
    );
}

#[test]
fn membership_preserves_value_across_try() {
    check_expression(
        "if (try ReadValue(1, FailAt)) in [1, try ReadValue(2, FailAt)] then return Result.Ok(12); end if; return Result.Ok(0);",
    );
}

#[test]
fn counting_loop_preserves_start_across_try_bound() {
    check_expression(
        "var Total: integer := 0; for I: integer := try ReadValue(1, FailAt) to try ReadValue(2, FailAt) do Total := Total * 10 + I; end for; return Result.Ok(Total);",
    );
}

#[test]
fn case_guard_preserves_subject_and_lower_comparison_across_try() {
    check_expression(
        "case 1 of when const Subject if (Subject >= (try ReadValue(1, FailAt))) and (Subject <= (try ReadValue(2, FailAt))): return Result.Ok(12); else return Result.Ok(0); end case;",
    );
}

#[test]
fn case_guard_preserves_subject_across_try() {
    check_expression(
        "case 1 of when const Subject if Subject = (try ReadValue(1, FailAt)): return Result.Ok(10 + (try ReadValue(2, FailAt))); else return Result.Ok(0); end case;",
    );
}

#[test]
fn record_argument_preserves_earlier_values_across_try() {
    check_expression(
        "var C: Counter := Counter(Base := 0); return Result.Ok(CounterAdd(C, try ReadValue(1, FailAt), try ReadValue(2, FailAt)));",
    );
}

#[test]
fn constructed_record_argument_survives_try() {
    check_expression(
        "return Result.Ok(CounterAdd(Counter(Base := 0), try ReadValue(1, FailAt), try ReadValue(2, FailAt)));",
    );
}

#[test]
fn procedure_record_argument_survives_try() {
    check_expression(
        "var C: Counter := Counter(Base := 0); CounterWriteNumber(C, (try ReadValue(1, FailAt)) * 10 + (try ReadValue(2, FailAt))); return Result.Ok(Written);",
    );
}

#[test]
fn optional_handler_preserves_arguments_across_try() {
    check_expression(
        "var C: Counter := Counter(Base := 0); C.OnValue := Option.Some(Combine); case C.OnValue of when Option.Some(const Handler): return Result.Ok(Handler(try ReadValue(1, FailAt), try ReadValue(2, FailAt))); when Option.None: return Result.Ok(0); end case;",
    );
}

#[test]
fn optional_handler_field_write_preserves_record_across_try() {
    check_expression(
        "var C: Counter := Counter(Base := 0); C.OnValue := Option.Some(try ReadHandler(FailAt)); case C.OnValue of when Option.Some(const Handler): return Result.Ok(Handler(1, 2)); when Option.None: return Result.Ok(0); end case;",
    );
}

#[test]
fn record_field_write_preserves_parent_across_try_index() {
    check_expression(
        "var Value: Bucket := Bucket(Items := [0, 0]); Value.Items[try ReadValue(1, FailAt)] := 12; var Ignored: integer := try ReadValue(2, FailAt); return Result.Ok(Value.Items[1]);",
    );
}

#[test]
fn option_try_preserves_arguments_and_stops_at_none() {
    assert_succeeds(
        r#"
program OptionArguments;
   var Trace: integer := 0;
function ReadValue(Value: integer; FailAt: integer): Option of (integer);
begin
  Trace := Trace * 10 + Value;
  if Value = FailAt then return Option.None; end if;
  return Option.Some(Value);
end function;
function Combine(X: integer; Y: integer): integer;
begin return X * 10 + Y; end function;
function Probe(FailAt: integer): Option of (integer);
begin return Option.Some(Combine(try ReadValue(1, FailAt), try ReadValue(2, FailAt))); end function;
begin
  if Probe(0) <> Option.Some(12) then panic('success'); end if;
  if Trace <> 12 then panic('order'); end if;
  Trace := 0;
  if Probe(1) <> Option.None then panic('first none'); end if;
  if Trace <> 1 then panic('evaluated after none'); end if;
  Trace := 0;
  if Probe(2) <> Option.None then panic('second none'); end if;
  if Trace <> 12 then panic('second order'); end if;
end program;
"#,
    );
}

#[test]
fn saved_operands_retain_snapshots_across_mutation_and_loop_iterations() {
    assert_succeeds(
        r#"
program Snapshots;
   var Values: array of (integer) := [12, 99];
function Change(): result of (integer, string);
begin Values := [77, 88]; return Result.Ok(0); end function;
function Probe(): result of (integer, string);
begin
  for I: integer := 1 to 3 do
  begin
    Values := [12, 99];
    if Values[try Change()] <> 12 then panic('collection snapshot'); end if;
     var X: integer := I;
    const Update: function(): result of (integer, string) := function(): result of (integer, string)
    begin X := 99; return Result.Ok(10); end function;
    if X + (try Update()) <> I + 10 then panic('local snapshot'); end if;
    if X <> 99 then panic('mutation missing'); end if;
  end; end for;
  return Result.Ok(12);
end function;
begin if Probe() <> Result.Ok(12) then panic('result'); end if; end program;
"#,
    );
}

#[test]
fn result_try_unwraps_generic_callee_to_concrete_payloads() {
    assert_succeeds(
        r#"program GenericResultTry;

function Tagged of (T)(Value: Result of (T, string)): Result of (T, string);
begin
  case Value of
    when Result.Ok(const Content):
      return Result.Ok(Content);
    when Result.Error(const Message):
      return Result.Error('tagged ' + Message);
  end case;
end function;

function MakeInt(Fail: boolean): Result of (integer, string);
begin
  if Fail then
    return Result.Error('int');
  end if;

  return Result.Ok(7);
end function;

function MakeValues(): Result of (array of (integer), string);
begin
  return Result.Ok([2, 3]);
end function;

function Probe(Fail: boolean): Result of (integer, string);
begin
  const Values: array of (integer) := try Tagged(MakeValues());
  const Number: integer := try Tagged(MakeInt(Fail));
  return Result.Ok(Number * 10 + Values[0] + Values[1]);
end function;

begin
  if Probe(false) <> Result.Ok(75) then
    panic('generic payloads');
  end if;

  if Probe(true) <> Result.Error('tagged int') then
    panic('generic error propagation');
  end if;
end program;
"#,
    );
}

#[test]
fn option_try_unwraps_generic_callee_to_concrete_payload() {
    assert_succeeds(
        r#"
program GenericOptionTry;
function Wrapped of (T)(Value: Option of (T)): Option of (T);
begin return Value; end function;
function Lookup(Present: boolean): Option of (integer);
begin
  if Present then return Option.Some(40); end if;
  return Option.None;
end function;
function Probe(Present: boolean): Option of (integer);
begin
  const Number: integer := try Wrapped(Lookup(Present));
  return Option.Some(Number + 2);
end function;
begin
  if Probe(true) <> Option.Some(42) then panic('generic payload'); end if;
  if Probe(false) <> Option.None then panic('generic none'); end if;
end program;
"#,
    );
}

use super::super::assert_succeeds;

fn check_expression(body: &str) {
    assert_succeeds(&format!(
        r#"
program TryExpressions;
uses Std.Tasks;
var Written: integer := 0;
var Grid: array of array of integer := [[0, 0, 0], [0, 0, 0]];
var Handler: Option of function(X: integer; Y: integer): integer := None;
type
  Binary = function(X: integer; Y: integer): integer;
  type Bucket = record Items: array of integer; end record;
  type Counter = record
    Base: integer;
    function Add(Self: Counter; X: integer; Y: integer): integer;
    begin return Self.Base + X * 10 + Y; end function;
    procedure WriteNumber(Self: Counter; Value: integer);
    begin Written := Self.Base + Value; end procedure;
    function ReadHandler(Self: Counter): Option of Binary;
    begin return Handler; end function;
    procedure WriteHandler(Self: Counter; Value: Option of Binary);
    begin Handler := Value; end procedure;
    event OnValue: function(X: integer; Y: integer): integer read ReadHandler write WriteHandler;
  end record;
  type Message = enum Move(X: integer; Y: integer); end enum;
  type Pair = record First: integer; Second: integer; end record;
var Trace: integer := 0;
function ReadValue(Value: integer; FailAt: integer): result of integer, string;
begin
  Trace := Trace * 10 + Value;
  if Value = FailAt then return Error('expected'); end if;
  return Ok(Value);
end function;
function Combine(X: integer; Y: integer): integer;
begin
  return X * 10 + Y;
end function;
function ReadHandler(FailAt: integer): result of Binary, string;
begin
  const X: integer := try ReadValue(1, FailAt);
  const Y: integer := try ReadValue(2, FailAt);
  return Ok(Combine);
end function;
function Probe(FailAt: integer): result of integer, string;
begin
  {body}
end function;
begin
  if Probe(0) <> Ok(12) then panic('success value'); end if;
  if Trace <> 12 then panic('success evaluation order'); end if;
  Trace := 0;
  if Probe(1) <> Error('expected') then panic('first error'); end if;
  if Trace <> 1 then panic('evaluated after first error'); end if;
  Trace := 0;
  if Probe(2) <> Error('expected') then panic('second error'); end if;
  if Trace <> 12 then panic('second error evaluation order'); end if;
end.
"#
    ));
}

#[test]
fn enum_constructor_preserves_earlier_try_arguments() {
    check_expression(
        "const Value: Message := Message.Move(try ReadValue(1, FailAt), try ReadValue(2, FailAt)); case Value of when Message.Move(const X, const Y): return Ok(X * 10 + Y); end case;",
    );
}

#[test]
fn direct_call_preserves_earlier_try_arguments() {
    check_expression("return Ok(Combine(try ReadValue(1, FailAt), try ReadValue(2, FailAt)));");
}

#[test]
fn array_literal_preserves_earlier_try_elements() {
    check_expression(
        "const Values: array of integer := [try ReadValue(1, FailAt), try ReadValue(2, FailAt)]; return Ok(Values[0] * 10 + Values[1]);",
    );
}

#[test]
fn dictionary_literal_preserves_try_key_across_try_value() {
    check_expression(
        "const Values: dict of integer to integer := [try ReadValue(1, FailAt): try ReadValue(2, FailAt)]; return Ok(10 + Values[1]);",
    );
}

#[test]
fn binary_expression_preserves_left_operand_across_try() {
    check_expression("return Ok((try ReadValue(1, FailAt)) * 10 + (try ReadValue(2, FailAt)));");
}

#[test]
fn record_update_preserves_base_and_fields_across_try() {
    check_expression(
        "const Original: Pair := Pair( First := 8, Second := 9 ); const Value: Pair := Original with First := try ReadValue(1, FailAt); Second := try ReadValue(2, FailAt); end with; return Ok(Value.First * 10 + Value.Second);",
    );
}

#[test]
fn function_value_preserves_callee_across_try() {
    check_expression(
        "const F: function(X: integer; Y: integer): integer := Combine; return Ok(F(try ReadValue(1, FailAt), try ReadValue(2, FailAt)));",
    );
}

#[test]
fn task_spawn_preserves_callee_and_arguments_across_try() {
    check_expression(
        "return Ok(Std.Tasks.Wait(go Combine(try ReadValue(1, FailAt), try ReadValue(2, FailAt))));",
    );
}

#[test]
fn index_read_preserves_collection_across_try() {
    check_expression(
        "const Values: array of integer := [0, 10]; return Ok(Values[try ReadValue(1, FailAt)] + (try ReadValue(2, FailAt)));",
    );
}

#[test]
fn nested_index_write_preserves_path_and_replacement_across_try() {
    check_expression(
        "var Values: array of array of integer := [[0, 0, 0], [0, 0, 0]]; Values[try ReadValue(1, FailAt)][try ReadValue(2, FailAt)] := 12; return Ok(Values[1][2]);",
    );
}

#[test]
fn global_index_write_preserves_path_and_replacement_across_try() {
    check_expression(
        "Grid[try ReadValue(1, FailAt)][try ReadValue(2, FailAt)] := 12; return Ok(Grid[1][2]);",
    );
}

#[test]
fn membership_preserves_value_across_try() {
    check_expression(
        "if (try ReadValue(1, FailAt)) in [1, try ReadValue(2, FailAt)] then return Ok(12); end if; return Ok(0);",
    );
}

#[test]
fn counting_loop_preserves_start_across_try_bound() {
    check_expression(
        "var Total: integer := 0; for I: integer := try ReadValue(1, FailAt) to try ReadValue(2, FailAt) do Total := Total * 10 + I; end for; return Ok(Total);",
    );
}

#[test]
fn case_guard_preserves_subject_and_lower_comparison_across_try() {
    check_expression(
        "case 1 of when const Subject if Subject >= (try ReadValue(1, FailAt)) and Subject <= (try ReadValue(2, FailAt)): return Ok(12); else return Ok(0); end case;",
    );
}

#[test]
fn case_guard_preserves_subject_across_try() {
    check_expression(
        "case 1 of when const Subject if Subject = (try ReadValue(1, FailAt)): return Ok(10 + (try ReadValue(2, FailAt))); else return Ok(0); end case;",
    );
}

#[test]
fn method_preserves_receiver_and_arguments_across_try() {
    check_expression(
        "const C: Counter := Counter( Base := 0 ); return Ok(C.Add(try ReadValue(1, FailAt), try ReadValue(2, FailAt)));",
    );
}

#[test]
fn postfix_method_preserves_receiver_across_try() {
    check_expression(
        "const C: Counter := Counter( Base := 0 ); return Ok((C).Add(try ReadValue(1, FailAt), try ReadValue(2, FailAt)));",
    );
}

#[test]
fn procedure_method_preserves_receiver_across_try() {
    check_expression(
        "const C: Counter := Counter( Base := 0 ); C.WriteNumber((try ReadValue(1, FailAt)) * 10 + (try ReadValue(2, FailAt))); return Ok(Written);",
    );
}

#[test]
fn event_raise_preserves_handler_and_arguments_across_try() {
    check_expression(
        "const C: Counter := Counter( Base := 0 ); C.OnValue := Combine; return Ok(C.OnValue(try ReadValue(1, FailAt), try ReadValue(2, FailAt)));",
    );
}

#[test]
fn event_write_preserves_receiver_across_try() {
    check_expression(
        "const C: Counter := Counter( Base := 0 ); C.OnValue := try ReadHandler(FailAt); return Ok(C.OnValue(1, 2));",
    );
}

#[test]
fn record_field_write_preserves_parent_across_try_index() {
    check_expression(
        "var Value: Bucket := Bucket( Items := [0, 0] ); Value.Items[try ReadValue(1, FailAt)] := 12; const Ignored: integer := try ReadValue(2, FailAt); return Ok(Value.Items[1]);",
    );
}

#[test]
fn option_try_preserves_arguments_and_stops_at_none() {
    assert_succeeds(
        r#"
program OptionArguments;
var Trace: integer := 0;
function ReadValue(Value: integer; FailAt: integer): Option of integer;
begin
  Trace := Trace * 10 + Value;
  if Value = FailAt then return None; end if;
  return Some(Value);
end function;
function Combine(X: integer; Y: integer): integer;
begin return X * 10 + Y; end function;
function Probe(FailAt: integer): Option of integer;
begin return Some(Combine(try ReadValue(1, FailAt), try ReadValue(2, FailAt))); end function;
begin
  if Probe(0) <> Some(12) then panic('success'); end if;
  if Trace <> 12 then panic('order'); end if;
  Trace := 0;
  if Probe(1) <> None then panic('first none'); end if;
  if Trace <> 1 then panic('evaluated after none'); end if;
  Trace := 0;
  if Probe(2) <> None then panic('second none'); end if;
  if Trace <> 12 then panic('second order'); end if;
end.
"#,
    );
}

#[test]
fn saved_operands_retain_snapshots_across_mutation_and_loop_iterations() {
    assert_succeeds(
        r#"
program Snapshots;
var Values: array of integer := [12, 99];
function Change(): result of integer, string;
begin Values := [77, 88]; return Ok(0); end function;
function Probe(): result of integer, string;
begin
  for I: integer := 1 to 3 do
  begin
    Values := [12, 99];
    if Values[try Change()] <> 12 then panic('collection snapshot'); end if;
    var X: integer := I;
    const Update: function(): result of integer, string := function(): result of integer, string
    begin X := 99; return Ok(10); end function;
    if X + (try Update()) <> I + 10 then panic('local snapshot'); end if;
    if X <> 99 then panic('mutation missing'); end if;
  end; end for;
  return Ok(12);
end function;
begin if Probe() <> Ok(12) then panic('result'); end if; end.
"#,
    );
}

#[test]
fn result_try_unwraps_generic_callee_to_concrete_payloads() {
    assert_succeeds(
        r#"
program GenericResultTry;
function Tagged<T>(Value: result of T, string): result of T, string;
begin
  case Value of
    when Ok(const Content): return Ok(Content);
    when Error(const Message): return Error('tagged ' + Message);
  end case;
end function;
function MakeInt(Fail: boolean): result of integer, string;
begin
  if Fail then return Error('int'); end if;
  return Ok(7);
end function;
function MakeValues(): result of array of integer, string;
begin return Ok([2, 3]); end function;
function Probe(Fail: boolean): result of integer, string;
begin
  const Values: array of integer := try Tagged(MakeValues());
  const Number: integer := try Tagged(MakeInt(Fail));
  return Ok(Number * 10 + Values[0] + Values[1]);
end function;
begin
  if Probe(false) <> Ok(75) then panic('generic payloads'); end if;
  if Probe(true) <> Error('tagged int') then panic('generic error propagation'); end if;
end.
"#,
    );
}

#[test]
fn option_try_unwraps_generic_callee_to_concrete_payload() {
    assert_succeeds(
        r#"
program GenericOptionTry;
function Wrapped<T>(Value: Option of T): Option of T;
begin return Value; end function;
function Lookup(Present: boolean): Option of integer;
begin
  if Present then return Some(40); end if;
  return None;
end function;
function Probe(Present: boolean): Option of integer;
begin
  const Number: integer := try Wrapped(Lookup(Present));
  return Some(Number + 2);
end function;
begin
  if Probe(true) <> Some(42) then panic('generic payload'); end if;
  if Probe(false) <> None then panic('generic none'); end if;
end.
"#,
    );
}

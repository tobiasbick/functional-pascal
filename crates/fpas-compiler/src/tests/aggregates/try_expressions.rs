use super::super::assert_succeeds;

fn check_expression(body: &str) {
    assert_succeeds(&format!(
        r#"
program TryExpressions;
uses Std.Task;
mutable var Written: integer := 0;
mutable var Grid: array of array of integer := [[0, 0, 0], [0, 0, 0]];
mutable var Handler: Option of function(X: integer; Y: integer): integer := None;
type
  Binary = function(X: integer; Y: integer): integer;
  Bucket = record Items: array of integer; end;
  Counter = record
    Base: integer;
    function Add(Self: Counter; X: integer; Y: integer): integer;
    begin return Self.Base + X * 10 + Y end;
    procedure WriteNumber(Self: Counter; Value: integer);
    begin Written := Self.Base + Value end;
    property Number: integer write WriteNumber;
    function ReadHandler(Self: Counter): Option of Binary;
    begin return Handler end;
    procedure WriteHandler(Self: Counter; Value: Option of Binary);
    begin Handler := Value end;
    event OnValue: function(X: integer; Y: integer): integer read ReadHandler write WriteHandler;
  end;
  Message = enum Move(X: integer; Y: integer); end;
  Pair = record First: integer; Second: integer; end;
mutable var Trace: integer := 0;
function ReadValue(Value: integer; FailAt: integer): result of integer, string;
begin
  Trace := Trace * 10 + Value;
  if Value = FailAt then return Error('expected');
  return Ok(Value)
end;
function Combine(X: integer; Y: integer): integer;
begin
  return X * 10 + Y
end;
function ReadHandler(FailAt: integer): result of Binary, string;
begin
  var X: integer := try ReadValue(1, FailAt);
  var Y: integer := try ReadValue(2, FailAt);
  return Ok(Combine)
end;
function Probe(FailAt: integer): result of integer, string;
begin
  {body}
end;
begin
  if Probe(0) <> Ok(12) then panic('success value');
  if Trace <> 12 then panic('success evaluation order');
  Trace := 0;
  if Probe(1) <> Error('expected') then panic('first error');
  if Trace <> 1 then panic('evaluated after first error');
  Trace := 0;
  if Probe(2) <> Error('expected') then panic('second error');
  if Trace <> 12 then panic('second error evaluation order')
end.
"#
    ));
}

#[test]
fn enum_constructor_preserves_earlier_try_arguments() {
    check_expression(
        "var Value: Message := Message.Move(try ReadValue(1, FailAt), try ReadValue(2, FailAt)); case Value of Message.Move(X, Y): return Ok(X * 10 + Y) end",
    );
}

#[test]
fn direct_call_preserves_earlier_try_arguments() {
    check_expression("return Ok(Combine(try ReadValue(1, FailAt), try ReadValue(2, FailAt)))");
}

#[test]
fn array_literal_preserves_earlier_try_elements() {
    check_expression(
        "var Values: array of integer := [try ReadValue(1, FailAt), try ReadValue(2, FailAt)]; return Ok(Values[0] * 10 + Values[1])",
    );
}

#[test]
fn dictionary_literal_preserves_try_key_across_try_value() {
    check_expression(
        "var Values: dict of integer to integer := [try ReadValue(1, FailAt): try ReadValue(2, FailAt)]; return Ok(10 + Values[1])",
    );
}

#[test]
fn binary_expression_preserves_left_operand_across_try() {
    check_expression("return Ok((try ReadValue(1, FailAt)) * 10 + (try ReadValue(2, FailAt)))");
}

#[test]
fn record_update_preserves_base_and_fields_across_try() {
    check_expression(
        "var Original: Pair := record First := 8; Second := 9; end; var Value: Pair := Original with First := try ReadValue(1, FailAt); Second := try ReadValue(2, FailAt); end; return Ok(Value.First * 10 + Value.Second)",
    );
}

#[test]
fn function_value_preserves_callee_across_try() {
    check_expression(
        "var F: function(X: integer; Y: integer): integer := Combine; return Ok(F(try ReadValue(1, FailAt), try ReadValue(2, FailAt)))",
    );
}

#[test]
fn task_spawn_preserves_callee_and_arguments_across_try() {
    check_expression(
        "return Ok(Std.Task.Wait(go Combine(try ReadValue(1, FailAt), try ReadValue(2, FailAt))))",
    );
}

#[test]
fn index_read_preserves_collection_across_try() {
    check_expression(
        "var Values: array of integer := [0, 10]; return Ok(Values[try ReadValue(1, FailAt)] + (try ReadValue(2, FailAt)))",
    );
}

#[test]
fn nested_index_write_preserves_path_and_replacement_across_try() {
    check_expression(
        "mutable var Values: array of array of integer := [[0, 0, 0], [0, 0, 0]]; Values[try ReadValue(1, FailAt)][try ReadValue(2, FailAt)] := 12; return Ok(Values[1][2])",
    );
}

#[test]
fn global_index_write_preserves_path_and_replacement_across_try() {
    check_expression(
        "Grid[try ReadValue(1, FailAt)][try ReadValue(2, FailAt)] := 12; return Ok(Grid[1][2])",
    );
}

#[test]
fn membership_preserves_value_across_try() {
    check_expression(
        "if (try ReadValue(1, FailAt)) in [1, try ReadValue(2, FailAt)] then return Ok(12); return Ok(0)",
    );
}

#[test]
fn counting_loop_preserves_start_across_try_bound() {
    check_expression(
        "mutable var Total: integer := 0; for I: integer := try ReadValue(1, FailAt) to try ReadValue(2, FailAt) do Total := Total * 10 + I; return Ok(Total)",
    );
}

#[test]
fn case_range_preserves_subject_and_lower_comparison_across_try() {
    check_expression(
        "case 1 of (try ReadValue(1, FailAt))..(try ReadValue(2, FailAt)): return Ok(12) else return Ok(0) end",
    );
}

#[test]
fn case_label_preserves_subject_across_try() {
    check_expression(
        "case 1 of (try ReadValue(1, FailAt)): return Ok(10 + (try ReadValue(2, FailAt))) else return Ok(0) end",
    );
}

#[test]
fn method_preserves_receiver_and_arguments_across_try() {
    check_expression(
        "var C: Counter := record Base := 0; end; return Ok(C.Add(try ReadValue(1, FailAt), try ReadValue(2, FailAt)))",
    );
}

#[test]
fn postfix_method_preserves_receiver_across_try() {
    check_expression(
        "var C: Counter := record Base := 0; end; return Ok((C).Add(try ReadValue(1, FailAt), try ReadValue(2, FailAt)))",
    );
}

#[test]
fn property_write_preserves_receiver_across_try() {
    check_expression(
        "var C: Counter := record Base := 0; end; C.Number := (try ReadValue(1, FailAt)) * 10 + (try ReadValue(2, FailAt)); return Ok(Written)",
    );
}

#[test]
fn event_raise_preserves_handler_and_arguments_across_try() {
    check_expression(
        "var C: Counter := record Base := 0; end; C.OnValue := Combine; return Ok(C.OnValue(try ReadValue(1, FailAt), try ReadValue(2, FailAt)))",
    );
}

#[test]
fn event_write_preserves_receiver_across_try() {
    check_expression(
        "var C: Counter := record Base := 0; end; C.OnValue := try ReadHandler(FailAt); return Ok(C.OnValue(1, 2))",
    );
}

#[test]
fn record_field_write_preserves_parent_across_try_index() {
    check_expression(
        "mutable var Value: Bucket := record Items := [0, 0]; end; Value.Items[try ReadValue(1, FailAt)] := 12; var Ignored: integer := try ReadValue(2, FailAt); return Ok(Value.Items[1])",
    );
}

#[test]
fn option_try_preserves_arguments_and_stops_at_none() {
    assert_succeeds(
        r#"
program OptionArguments;
mutable var Trace: integer := 0;
function ReadValue(Value: integer; FailAt: integer): Option of integer;
begin
  Trace := Trace * 10 + Value;
  if Value = FailAt then return None;
  return Some(Value)
end;
function Combine(X: integer; Y: integer): integer;
begin return X * 10 + Y end;
function Probe(FailAt: integer): Option of integer;
begin return Some(Combine(try ReadValue(1, FailAt), try ReadValue(2, FailAt))) end;
begin
  if Probe(0) <> Some(12) then panic('success');
  if Trace <> 12 then panic('order');
  Trace := 0;
  if Probe(1) <> None then panic('first none');
  if Trace <> 1 then panic('evaluated after none');
  Trace := 0;
  if Probe(2) <> None then panic('second none');
  if Trace <> 12 then panic('second order')
end.
"#,
    );
}

#[test]
fn saved_operands_retain_snapshots_across_mutation_and_loop_iterations() {
    assert_succeeds(
        r#"
program Snapshots;
mutable var Values: array of integer := [12, 99];
function Change(): result of integer, string;
begin Values := [77, 88]; return Ok(0) end;
function Probe(): result of integer, string;
begin
  for I: integer := 1 to 3 do
  begin
    Values := [12, 99];
    if Values[try Change()] <> 12 then panic('collection snapshot');
    mutable var X: integer := I;
    var Update: function(): result of integer, string := function(): result of integer, string
    begin X := 99; return Ok(10) end;
    if X + (try Update()) <> I + 10 then panic('local snapshot');
    if X <> 99 then panic('mutation missing')
  end;
  return Ok(12)
end;
begin if Probe() <> Ok(12) then panic('result') end.
"#,
    );
}

use super::assert_both_succeed;

#[test]
fn direct_function_call_matches_stack_path() {
    assert_both_succeed(
        r#"
program DirectCall;
function Add(A: integer; B: integer): integer;
begin
  return A + B;
end;
begin
  if Add(20, 22) <> 42 then
    panic('direct call mismatch');
end.
"#,
    );
}

#[test]
fn recursion_and_early_return_match_stack_path() {
    assert_both_succeed(
        r#"
program RecursiveCall;
function Factorial(N: integer): integer;
begin
  if N <= 1 then
    return 1;
  return N * Factorial(N - 1);
end;
begin
  if Factorial(5) <> 120 then
    panic('recursion mismatch');
end.
"#,
    );
}

#[test]
fn procedure_call_matches_stack_path() {
    assert_both_succeed(
        r#"
program ProcedureCall;
procedure Validate(Value: integer);
begin
  if Value <> 42 then
    panic('procedure argument mismatch');
end;
begin
  Validate(42);
end.
"#,
    );
}

#[test]
fn noncapturing_nested_routine_uses_numeric_id() {
    assert_both_succeed(
        r#"
program NestedCall;
function Outer(Value: integer): integer;
  function Double(Input: integer): integer;
  begin
    return Input + Input;
  end;
begin
  return Double(Value);
end;
begin
  if Outer(21) <> 42 then
    panic('nested call mismatch');
end.
"#,
    );
}

#[test]
fn first_class_named_function_uses_call_value() {
    assert_both_succeed(
        r#"
program FirstClassFunction;
function Double(Value: integer): integer;
begin
  return Value + Value;
end;
function Apply(Action: function(Value: integer): integer; Value: integer): integer;
begin
  return Action(Value);
end;
begin
  if Apply(Double, 21) <> 42 then
    panic('function value mismatch');
end.
"#,
    );
}

#[test]
fn first_class_named_procedure_uses_call_value() {
    assert_both_succeed(
        r#"
program FirstClassProcedure;
procedure Validate(Value: integer);
begin
  if Value <> 42 then
    panic('procedure callback mismatch');
end;
procedure Invoke(Action: procedure(Value: integer); Value: integer);
begin
  Action(Value);
end;
begin
  Invoke(Validate, 42);
end.
"#,
    );
}

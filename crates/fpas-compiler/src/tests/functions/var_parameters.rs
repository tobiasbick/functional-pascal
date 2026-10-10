//! `var` parameters: writes reach the caller's variable, field, or element as they execute.
//!
//! Documentation: `docs/pascal/language/functions/var-parameters.md`

use super::super::assert_succeeds;

mod named_arguments;
mod transitive_captures;

#[test]
fn var_parameters_update_variables_fields_elements_and_globals() {
    assert_succeeds(
        r#"
program VarTargets;
type Point = record
  X: integer;
  Y: integer;
end record;
var Global: integer := 100;
procedure Increase(var Value: integer);
begin
  Value := Value + 1;
end procedure;
procedure Swap(var A: integer; var B: integer);
begin
  const T: integer := A;
  A := B;
  B := T;
end procedure;
procedure MovePoint(var P: Point);
begin
  P.X := P.X + 10;
  Increase(var P.Y);
end procedure;
begin
  var Counter: integer := 0;
  Increase(var Counter);
  var A: integer := 1;
  var B: integer := 2;
  Swap(var A, var B);
  var P: Point := Point( X := 1, Y := 2 );
  MovePoint(var P);
  Increase(var P.X);
  var Items: array of integer := [5, 6, 7];
  Increase(var Items[1]);
  Increase(var Global);
  Swap(var Items[0], var Counter);
  if (Counter <> 5) or (Items[0] <> 1) or (A <> 2) or (B <> 1) then
    panic('variables'); end if;
  if (P.X <> 12) or (P.Y <> 3) or (Items[1] <> 7) or (Global <> 101) then
    panic('fields, elements, or globals'); end if;
end.
"#,
    );
}

#[test]
fn var_parameters_forward_and_reach_nested_routines_and_function_values() {
    assert_succeeds(
        r#"
program VarForwarding;
procedure Increase(var Value: integer);
begin
  Value := Value + 1;
end procedure;
procedure Twice(var Value: integer);
  procedure Bump();
  begin
    Increase(var Value);
  end procedure;
begin
  Bump();
  Increase(var Value);
end procedure;
procedure Swap<T>(var A: T; var B: T);
begin
  const Temp: T := A;
  A := B;
  B := Temp;
end procedure;
begin
  var Counter: integer := 0;
  Twice(var Counter);
  const F: procedure(var Value: integer) := Increase;
  F(var Counter);
  const G: procedure(var Value: integer) := procedure(var Value: integer) begin
    Value := Value * 10;
  end procedure;
  G(var Counter);
  var Shared: integer := 0;
  const Inc: procedure() := procedure() begin
    Increase(var Shared);
  end procedure;
  Inc();
  var Left: string := 'a';
  var Right: string := 'b';
  Swap(var Left, var Right);
  if (Counter <> 30) or (Shared <> 1) or (Left <> 'b') or (Right <> 'a') then
    panic('forwarding'); end if;
end.
"#,
    );
}

#[test]
fn var_arguments_evaluate_roots_and_indices_once_in_written_order() {
    assert_succeeds(
        r#"
program VarOrder;
var Trace: string := '';
function Mark(Tag: string; Value: integer): integer;
begin
  Trace := Trace + Tag;
  return Value;
end function;
procedure Store(First: integer; var Target: integer; Last: integer);
begin
  Target := First + Last;
end procedure;
begin
  var Items: array of integer := [0, 0, 0];
  Store(Mark('a', 1), var Items[Mark('b', 2)], Mark('c', 3));
  if (Trace <> 'abc') or (Items[2] <> 4) then
    panic('evaluation order'); end if;
end.
"#,
    );
}

#[test]
fn writes_before_a_try_exit_remain_visible() {
    assert_succeeds(
        r#"
program VarTry;
function Partial(var Value: integer): Result of (integer, string);
begin
  Value := 1;
  return Error('stopped');
end function;
function Run(var Value: integer): Result of (integer, string);
begin
  const Ignored: integer := try Partial(var Value);
  Value := 99;
  return Ok(Ignored);
end function;
begin
  var Value: integer := 0;
  const Outcome: Result of (integer, string) := Run(var Value);
  if Value <> 1 then
    panic('write before try exit'); end if;
end.
"#,
    );
}

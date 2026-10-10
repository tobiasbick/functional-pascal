//! Named reference arguments bind by parameter name and evaluate in written order.
//!
//! Documentation: `docs/pascal/language/functions/var-parameters.md`

use super::assert_succeeds;

#[test]
fn named_var_arguments_evaluate_indices_once_and_pass_in_parameter_order() {
    assert_succeeds(
        r#"
program NamedVarOrder;
var Trace: string := '';
function Mark(Tag: string; Value: integer): integer;
begin Trace := Trace + Tag; return Value; end function;
procedure Store(First: integer; var Target: integer; Last: integer);
begin Target := First * 10 + Last; end procedure;
procedure Assign(var Left: integer; var Right: integer; Value: integer);
begin Left := Value; Right := Value + 1; end procedure;
begin
  var Items: array of integer := [0, 0, 0];
  Store(Last := Mark('c', 3), Target := var Items[Mark('b', 2)], First := Mark('a', 1));
  if (Trace <> 'cba') or (Items[2] <> 13) then panic('order or index evaluation'); end if;
  Trace := '';
  var Other: array of integer := [0];
  Assign(Right := var Items[Mark('r', 1)], Value := Mark('v', 40), Left := var Other[Mark('l', 0)]);
  if (Trace <> 'rvl') or (Other[0] <> 40) or (Items[1] <> 41) then panic('reference mapping'); end if;
end.
"#,
    );
}

#[test]
fn named_var_arguments_support_generics_forwarding_members_and_captures() {
    assert_succeeds(
        r#"
program NamedVarMembers;
type Point = record
  X: integer;
  procedure Add(Self: Point; Step: integer; var Target: integer);
  begin Target := Target + Self.X + Step; end procedure;
  static procedure SetValue(Value: integer; var Target: integer);
  begin Target := Value; end procedure;
end record;
function Origin(): Point; begin return Point( X := 3 ); end function;
procedure Increase(var Value: integer); begin Value := Value + 1; end procedure;
procedure Forward(var Value: integer);
  procedure Nested(); begin Increase(Value := var Value); end procedure;
begin Nested(); Increase(Value := var Value); end procedure;
procedure Swap<T>(var A: T; var B: T);
begin const Temp: T := A; A := B; B := Temp; end procedure;
var Global: integer := 10;
begin
  var Counter: integer := 0;
  Forward(Value := var Counter);
  const Inc: procedure() := procedure() begin Increase(Value := var Counter); end procedure;
  Inc();
  const P: Point := Origin();
  P.Add(Target := var Counter, Step := 4);
  Origin().Add(Target := var Counter, Step := 2);
  Point.SetValue(Target := var Global, Value := Counter);
  var Left: string := 'a'; var Right: string := 'b';
  Swap(B := var Right, A := var Left);
  if (Counter <> 15) or (Global <> 15) or (Left <> 'b') or (Right <> 'a') then panic('named references'); end if;
end.
"#,
    );
}

#[test]
fn named_var_writes_to_variables_fields_and_elements_survive_try_exit() {
    assert_succeeds(
        r#"
program NamedVarTry;
type Point = record X: integer; end record;
function Partial(var Value: integer): Result of (integer, string);
begin Value := 7; return Error('stopped'); end function;
function Run(var Target: integer): Result of (integer, string);
begin
  const Ignored: integer := try Partial(Value := var Target);
  Target := 99; return Ok(Ignored);
end function;
begin
  var Value: integer := 0;
  var P: Point := Point( X := 0 );
  var Items: array of integer := [0];
  const A: Result of (integer, string) := Run(Target := var Value);
  const B: Result of (integer, string) := Run(Target := var P.X);
  const C: Result of (integer, string) := Run(Target := var Items[0]);
  if (Value <> 7) or (P.X <> 7) or (Items[0] <> 7) then panic('writes before try exit'); end if;
end.
"#,
    );
}

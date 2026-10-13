//! Named call arguments: written-order evaluation, parameter-order passing.
//!
//! Documentation: `docs/pascal/language/functions/parameters.md`

use super::super::assert_succeeds;

#[test]
fn named_arguments_evaluate_in_written_order_and_bind_by_name() {
    assert_succeeds(
        r#"
program NamedOrder;
var Trace: string := '';
function Mark(Tag: string; Value: integer): integer;
begin
  Trace := Trace + Tag;
  return Value;
end function;
function Sub(Left: integer; Right: integer): integer;
begin
  return Left - Right;
end function;
procedure Check(Expected: integer; Actual: integer);
begin
  if Expected <> Actual then
    panic('named argument mismatch'); end if;
end procedure;
begin
  const Value: integer := Sub(Right := Mark('r', 1), Left := Mark('l', 10));
  if Value <> 9 then
    panic('named arguments bound to the wrong parameters'); end if;
  if Trace <> 'rl' then
    panic('named arguments were not evaluated in written order'); end if;
  Check(Actual := Sub(Left := 5, Right := 2), Expected := 3);
end.
"#,
    );
}

#[test]
fn named_callable_values_use_static_names_and_written_order() {
    assert_succeeds(
        r#"
program NamedCallable;
uses Std.Math, Std.Tasks;
var Trace: string := '';
function Mark(Tag: string; Value: integer): integer;
begin
  Trace := Trace + Tag;
  return Value;
end function;
function Sub(Left: integer; Right: integer): integer;
begin
  return Left - Right;
end function;
function Apply(F: function(A: integer; B: integer): integer): integer;
begin
  return F(B := Mark('b', 2), A := Mark('a', 12));
end function;
begin
  const F: function(X: integer; Y: integer): integer := Sub;
  if F(Y := Mark('y', 3), X := Mark('x', 20)) <> 17 then panic('static slots'); end if;
  if Apply(F) <> 10 then panic('callback slots'); end if;
  const G := if true then Sub else function(First: integer; Second: integer): integer
    begin return First + Second; end function end if;
  if G(Right := 2, Left := 9) <> 7 then panic('inferred signature'); end if;
  const Jobs := [F];
  if Jobs[0](Y := 2, X := 8) <> 6 then panic('aggregate signature'); end if;
  if Trace <> 'yxba' then panic('written order'); end if;
  const Handle := go F(Y := 2, X := 30);
  if Wait(Handle := Handle) <> 28 then panic('named task'); end if;
  if Clamp(Hi := 8, V := 12, Lo := 3) <> 8 then panic('polymorphic slots'); end if;
  if Abs(N := -4) <> 4 then panic('polymorphic single argument'); end if;
  const MathJob := go Clamp(Hi := 8, V := 12, Lo := 3);
  if Wait(Handle := MathJob) <> 8 then panic('polymorphic intrinsic task'); end if;
  const Queue: channel of integer := CreateChannel(Capacity := 1);
  const Sent := Send(Value := 42, Queue := Queue);
  if not (Sent is Ok(Value := true)) then panic('named send'); end if;
  const Received := Receive(Queue := Queue);
  if not (Received is Ok(Value := 42)) then panic('named receive'); end if;
  discard CloseChannel(Queue := Queue);
  const Wrapped: option of channel of integer := Some(Value := CreateChannel(Capacity := 1));
  const Inner := Wrapped.Unwrap();
  discard Send(Value := 7, Queue := Inner).Unwrap();
  if Receive(Queue := Inner).Unwrap() <> 7 then panic('named payload context'); end if;
  discard CloseChannel(Queue := Inner);
end.
"#,
    );
}

#[test]
fn named_nested_patterns_extract_declared_fields_and_preserve_coverage() {
    assert_succeeds(
        r#"
program NamedPatterns;
type Pair of T = enum
  Values(First: T; Second: T);
  Empty;
end enum;
var Reads: integer := 0;
function Input(): Pair of option of integer;
begin
  Reads := Reads + 1;
  return Pair.Values(Second := Some(Value := 3), First := Some(Value := 7));
end function;
begin
  if not (Input() is Pair.Values(Second := Some(Value := 3), First := Some(7))) then
    panic('named is fields'); end if;
  if Reads <> 1 then panic('selector evaluated twice'); end if;
  case Input() of
    when Pair.Values(Second := Some(Value := const B), First := Some(const A)):
      if (A <> 7) or (B <> 3) then panic('named binding slots'); end if;
    when Pair.Values(First := _, Second := _), Pair.Empty: panic('wrong case');
  end case;
  const Outcome: result of (integer, Pair of integer) := Error(Value := Pair.Values(4, 9));
  case Outcome of
    when Error(Value := Pair.Values(Second := const B, First := 4)):
      if B <> 9 then panic('error payload'); end if;
    when Ok(Value := _), Error(Value := _): panic('wrong result');
  end case;
end.
"#,
    );
}

#[test]
fn named_arguments_apply_to_methods_static_routines_generics_and_go() {
    assert_succeeds(
        r#"
program NamedMembers;
uses Std.Tasks;
type Point = record
  X: integer;
  Y: integer;

  function Moved(Self: Point; Dx: integer; Dy: integer): Point;
  begin
    return Self with X := Self.X + Dx; Y := Self.Y + Dy; end with;
  end function;

  static function Create(X: integer; Y: integer): Point;
  begin
    return Point( X := X, Y := Y );
  end function;
end record;
function Origin(): Point;
begin
  return Point.Create(Y := 0, X := 0);
end function;
function Pick<T>(First: T; Second: T): T;
begin
  return Second;
end function;
function Work(Base: integer; Factor: integer): integer;
begin
  return Base * Factor;
end function;
begin
  const P: Point := Point.Create(Y := 2, X := 1);
  const Q: Point := P.Moved(Dy := 10, Dx := 100);
  if (Q.X <> 101) or (Q.Y <> 12) then
    panic('method named arguments'); end if;
  const R: Point := Origin().Moved(Dy := 1, Dx := 2);
  if (R.X <> 2) or (R.Y <> 1) then
    panic('postfix method named arguments'); end if;
  if Pick(Second := 'b', First := 'a') <> 'b' then
    panic('generic named arguments'); end if;
  const Bound: function(Horizontal: integer; Vertical: integer): Point := P.Moved;
  const B := Bound(Vertical := 30, Horizontal := 40);
  if (B.X <> 41) or (B.Y <> 32) then panic('bound static signature'); end if;
  const T: task of integer := go Work(Factor := 7, Base := 6);
  if Std.Tasks.Wait(T) <> 42 then
    panic('go named arguments'); end if;
end.
"#,
    );
}

#[test]
fn named_arguments_apply_to_standard_library_routines() {
    assert_succeeds(
        r#"
program NamedStd;

begin
  if 'x'.PadLeft(PadChar := '.', Width := 3) <> '..x' then
    panic('standard-library named arguments'); end if;
end.
"#,
    );
}

#[test]
fn named_variant_fields_evaluate_in_written_order() {
    assert_succeeds(
        r#"
program NamedVariants;
type Shape = enum
  Circle(Radius: real);
  Rect(Width: real; Height: real);
end enum;
var Trace: string := '';
function Mark(Tag: string; Value: real): real;
begin
  Trace := Trace + Tag;
  return Value;
end function;
begin
  const S: Shape := Shape.Rect(Height := Mark('h', 1.0), Width := Mark('w', 10.0));
  case S of
    when Shape.Rect(const W, const H):
      if (W <> 10.0) or (H <> 1.0) then
        panic('named variant fields bound to the wrong fields'); end if;
    when Shape.Circle(const R): panic('wrong variant');
  end case;
  if Trace <> 'hw' then
    panic('named variant fields were not evaluated in written order'); end if;
  if Circle(Radius := 2.0) <> Shape.Circle(2.0) then
    panic('short named variant construction'); end if;
end.
"#,
    );
}

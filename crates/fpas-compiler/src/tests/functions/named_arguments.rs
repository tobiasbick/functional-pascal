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
    return record X := X; Y := Y; end;
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
  const T: task := go Work(Factor := 7, Base := 6);
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
uses Std.Str;
begin
  if Std.Str.PadLeft(PadChar := '.', Width := 3, S := 'x') <> '..x' then
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
    when Shape.Rect(W, H):
      if (W <> 10.0) or (H <> 1.0) then
        panic('named variant fields bound to the wrong fields'); end if;
    when Shape.Circle(R): panic('wrong variant');
  end case;
  if Trace <> 'hw' then
    panic('named variant fields were not evaluated in written order'); end if;
  if Circle(Radius := 2.0) <> Shape.Circle(2.0) then
    panic('short named variant construction'); end if;
end.
"#,
    );
}

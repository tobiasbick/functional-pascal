//! Structural `=` and `<>` on records and payload enums at runtime.

use super::super::assert_succeeds;

#[test]
fn equatable_generics_execute_structural_comparison_and_error_construction() {
    assert_succeeds(
        r#"
program EqualityGenerics;
type Point = record X: integer; end record;
type Tree = enum Leaf(Value: Point); Branch(Child: option of Tree); end enum;
type Marker of T: Equatable = error Failed; end error;
function Same<T: Equatable>(A: T; B: T): boolean; begin return A = B; end function;
function Find<K: Equatable, V>(D: dict of K to V; Key: K): option of V; begin return D.Get(Key); end function;
function Mark<T: Equatable>(Value: T): Marker of T; begin return Marker.Failed; end function;
begin
  const P := Point(X := 4);
  if not Same(P, Point(X := 4)) or Same(P, Point(X := 3)) then panic('record equality'); end if;
  const T := Tree.Branch(Some(Tree.Leaf(P)));
  if not Same(T, T) then panic('recursive equality'); end if;
  const D: dict of Point to string := [P: 'value'];
  if Find(D, P) <> Some('value') then panic('record key'); end if;
  const M := Mark(P);
  if not (M is Marker.Failed) then panic('record constructor metadata'); end if;
end.
"#,
    );
}

#[test]
fn records_and_payload_enums_compare_structurally() {
    assert_succeeds(
        r#"
program Eq2;
type
  Point = record
    X: integer;
    Y: real;
  end record;
  type Box = record
    Corner: Point;
    Label: string;
    Tag: option of Point;
  end record;
  type Shape = enum
    Circle(Center: Point; Radius: integer);
    Square(Side: integer);
    Dot;
  end enum;
procedure Check(Name: string; Value: boolean; Expected: boolean);
begin
  if Value <> Expected then panic('wrong: ' + Name); end if;
end procedure;
begin
  const A: Point := Point( X := 1, Y := 2.0 );
  const B: Point := Point( X := 1, Y := 2.0 );
  const C: Point := A with Y := 2.5; end with;
  Check('same fields', A = B, true);
  Check('updated field', A = C, false);
  Check('not equal', A <> C, true);
  const Box1: Box := Box( Corner := A, Label := 'a', Tag := Some(B) );
  const Box2: Box := Box( Corner := B, Label := 'a', Tag := Some(A) );
  Check('nested', Box1 = Box2, true);
  Check('nested differs', Box1 = (Box2 with Tag := None; end with), false);
  const S1: Shape := Shape.Circle(A, 3);
  Check('same variant payload', S1 = Shape.Circle(B, 3), true);
  Check('same variant other payload', S1 = Shape.Circle(A, 4), false);
  Check('other variant', S1 = Shape.Square(3), false);
  Check('unit variant', Shape.Dot = Shape.Dot, true);
  Check('unit vs payload', Shape.Dot <> S1, true);
end.
"#,
    );
}

//! Structural `=` and `<>` on records and payload enums at runtime.

use super::super::assert_succeeds;

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
  Box = record
    Corner: Point;
    Label: string;
    Tag: option of Point;
  end record;
  Shape = enum
    Circle(Center: Point; Radius: integer);
    Square(Side: integer);
    Dot;
  end enum;
procedure Check(Name: string; Value: boolean; Expected: boolean);
begin
  if Value <> Expected then panic('wrong: ' + Name); end if;
end procedure;
begin
  var A: Point := record X := 1; Y := 2.0; end;
  var B: Point := record X := 1; Y := 2.0; end;
  var C: Point := A with Y := 2.5; end with;
  Check('same fields', A = B, true);
  Check('updated field', A = C, false);
  Check('not equal', A <> C, true);
  var Box1: Box := record Corner := A; Label := 'a'; Tag := Some(B); end;
  var Box2: Box := record Corner := B; Label := 'a'; Tag := Some(A); end;
  Check('nested', Box1 = Box2, true);
  Check('nested differs', Box1 = (Box2 with Tag := None; end with), false);
  var S1: Shape := Shape.Circle(A, 3);
  Check('same variant payload', S1 = Shape.Circle(B, 3), true);
  Check('same variant other payload', S1 = Shape.Circle(A, 4), false);
  Check('other variant', S1 = Shape.Square(3), false);
  Check('unit variant', Shape.Dot = Shape.Dot, true);
  Check('unit vs payload', Shape.Dot <> S1, true);
end.
"#,
    );
}

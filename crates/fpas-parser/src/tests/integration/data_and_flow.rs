use super::*;

#[test]
fn record_creation_and_access() {
    let p = parse_ok(
        "\
program Geometry;

type Point = record
  X: real;
  Y: real;
end;

begin
  var P: Point := record X := 1.0; Y := 2.0; end;
  var Sum: real := P.X + P.Y
end.",
    );
    assert_eq!(p.declarations.len(), 1);
    assert_eq!(p.body.len(), 2);
}

#[test]
fn nested_loops() {
    let p = parse_ok(
        "\
program T;
begin
  for I: integer := 0 to 9 do
    for J: integer := 0 to 9 do
      begin
        var X: integer := I * 10 + J;
        if X mod 2 = 0 then
          continue
      end
end.",
    );
    assert_eq!(p.body.len(), 1);
    match &p.body[0] {
        Stmt::For { body, .. } => {
            assert!(matches!(body.as_ref(), Stmt::For { .. }));
        }
        _ => panic!("expected nested For"),
    }
}

#[test]
fn repeat_with_break() {
    let p = parse_ok(
        "\
program T;
begin
  mutable var X: integer := 0;
  repeat
    X := X + 1;
    if X = 10 then break
  until X = 100
end.",
    );
    assert_eq!(p.body.len(), 2);
}

#[test]
fn array_operations() {
    let p = parse_ok(
        "\
program T;
begin
  var Xs: array of integer := [1, 2, 3, 4, 5];
  var First: integer := Xs[0];
  var Last: integer := Xs[4]
end.",
    );
    assert_eq!(p.body.len(), 3);
}
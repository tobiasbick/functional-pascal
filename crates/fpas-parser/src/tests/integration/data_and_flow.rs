use super::*;

#[test]
fn record_creation_and_access() {
    let p = parse_ok(
        r#"program Geometry;

type Point = record
  X: real;
  Y: real;
end record;

begin
  const P: Point := Point(X := 1.0, Y := 2.0);
  const Sum: real := P.X + P.Y;
end program;
"#,
    );
    assert_eq!(p.declarations.len(), 1);
    assert_eq!(p.body.len(), 2);
}

#[test]
fn nested_loops() {
    let p = parse_ok(
        r#"program T;
begin
  for I: integer := 0 to 9 do
    for J: integer := 0 to 9 do
      begin
        const X: integer := I * 10 + J;
        if X mod 2 = 0 then
          continue; end if;
      end; end for; end for;
end program;"#,
    );
    assert_eq!(p.body.len(), 1);
    match &p.body[0] {
        Stmt::For { body, .. } => {
            assert!(matches!(first(body), Stmt::For { .. }));
        }
        _ => panic!("expected nested For"),
    }
}

#[test]
fn repeat_with_break() {
    let p = parse_ok(
        r#"program T;
begin
   var X: integer := 0;
  repeat
    X := X + 1;
    if X = 10 then break; end if;
  until X = 100;
end program;"#,
    );
    assert_eq!(p.body.len(), 2);
}

#[test]
fn array_operations() {
    let p = parse_ok(
        r#"program T;
begin
  const Xs: array of (integer) := [1, 2, 3, 4, 5];
  const First: integer := Xs[0];
  const Last: integer := Xs[4];
end program;"#,
    );
    assert_eq!(p.body.len(), 3);
}

fn first(statement: &Stmt) -> &Stmt {
    let Stmt::StatementList(statements, _) = statement else {
        panic!("expected statement list");
    };
    &statements[0]
}

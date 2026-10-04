use super::{check_errors, check_ok};

const CONTEXTUAL_RECORD_SOURCE: &str = r#"program T;

type Point = record
  X: integer := 0;
  Y: integer := 0;
end record;

const OriginPoint: Point := Point(X := 0);

function Origin(): Point;
begin
  return Point(X := 0);
end function;

procedure Draw(P: Point);
begin
  null;
end procedure;

begin
  mutable var P: Point := Point();
  P := Point(X := 1);
  Draw(Point(Y := 2));
  var Points: array of (Point) := [Point(X := 3)];
end program;
"#;

#[test]
fn record_constructions_use_expected_types_in_all_contexts() {
    check_ok(CONTEXTUAL_RECORD_SOURCE);
}

#[test]
fn contextual_record_construction_still_requires_non_defaulted_fields() {
    let errors = check_errors(
        r#"program T;

type Point = record
  X: integer;
  Y: integer := 0;
end record;

begin
  var P: Point := Point(Y := 1);
end program;
"#,
    );

    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_MISSING_RECORD_FIELD),
        "{errors:#?}"
    );
}

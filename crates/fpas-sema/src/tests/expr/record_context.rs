use super::{check_errors, check_ok};

const CONTEXTUAL_RECORD_SOURCE: &str = r#"
program T;
 type Point = record
  X: integer := 0;
  Y: integer := 0;
end record;
 const OriginPoint: Point := record X := 0; end record;
function Origin(): Point;
begin
  return record X := 0; end record;
end function;
procedure Draw(P: Point);
begin null;
end procedure;
begin
  mutable var P: Point := record end record;
  P := record X := 1; end record;
  Draw(record Y := 2; end record);
  var Points: array of Point := [record X := 3; end record];
end program;
"#;

#[test]
fn record_literals_use_expected_types_in_all_contexts() {
    check_ok(CONTEXTUAL_RECORD_SOURCE);
}

#[test]
fn contextual_record_literal_still_requires_non_defaulted_fields() {
    let errors = check_errors(
        r#"program T;  type Point = record X: integer; Y: integer := 0; end record; begin var P: Point := record Y := 1; end record; end program;"#,
    );

    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_MISSING_RECORD_FIELD),
        "{errors:#?}"
    );
}

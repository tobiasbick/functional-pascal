use super::check_errors;
use crate::{SemaError, analyze_unit};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};
use fpas_parser::{CompilationUnit, Unit, parse_compilation_unit};

const PREFIX: &str = "program T; type Point = record X: integer; end record; begin const P: Point := record X := 1; end;";

#[test]
fn record_update_rejects_non_record_base() {
    let errors = check_errors(
        "program T; begin const X: integer := 1; const Y: integer := X with Value := 2; end with; end.",
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "{errors:#?}"
    );
}

#[test]
fn record_update_rejects_unknown_and_wrongly_typed_fields() {
    for (update, expected_code) in [
        ("P with Missing := 2; end with", SEMA_UNKNOWN_NAME),
        ("P with X := 'wrong'; end with", SEMA_TYPE_MISMATCH),
    ] {
        let source = format!("{PREFIX} const Q: Point := {update}; end.");
        let errors = check_errors(&source);
        assert!(
            errors.iter().any(|error| error.code == expected_code),
            "update: {update}; errors: {errors:#?}"
        );
    }
}

fn parse_unit(source: &str) -> Unit {
    let (parsed, errors) = parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:#?}");
    let CompilationUnit::Unit(unit) = parsed else {
        panic!("fixture must be a unit");
    };
    unit
}

fn imported_update_errors(update: &str) -> Vec<SemaError> {
    let types = parse_unit(
        "unit Demo.Types;
         public type Point = record public X: integer; public Y: integer; end record;
         public type OtherPoint = record public X: integer; public Y: integer; end record;
         public type Holder = record
           public Position: Point;
           public Points: array of Point;
         end record;\nend unit;",
    );
    let types_analysis = analyze_unit(&types, &[]).expect("type unit analysis");
    assert!(types_analysis.metadata.errors.is_empty());
    let interface = types_analysis.interface.expect("type unit interface");
    let consumer = parse_unit(&format!(
        "unit Demo.Consumer; uses Demo.Types;
         public function Change(Original: Holder; Other: OtherPoint): Holder;
         begin return Original with {update}; end with; end function;\nend unit;"
    ));
    analyze_unit(&consumer, &[interface])
        .expect("consumer analysis")
        .metadata
        .errors
}

#[test]
fn record_update_contextually_types_imported_record_literals_and_array_elements() {
    for update in [
        "Position := record X := 3; Y := 4; end",
        "Points := [record X := 3; Y := 4; end]",
    ] {
        let errors = imported_update_errors(update);
        assert!(errors.is_empty(), "update: {update}; errors: {errors:#?}");
    }
}

#[test]
fn record_update_preserves_imported_record_field_and_nominal_validation() {
    for (update, code) in [
        (
            "Position := record X := 'wrong'; Y := 4; end",
            SEMA_TYPE_MISMATCH,
        ),
        (
            "Position := record X := 3; Y := 4; Missing := 0; end",
            SEMA_UNKNOWN_NAME,
        ),
        ("Position := Other", SEMA_TYPE_MISMATCH),
        ("Points := [Other]", SEMA_TYPE_MISMATCH),
    ] {
        let errors = imported_update_errors(update);
        assert!(
            errors.iter().any(|error| error.code == code),
            "update: {update}; errors: {errors:#?}"
        );
    }
}

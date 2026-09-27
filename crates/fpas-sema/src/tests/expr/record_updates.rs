use super::check_errors;
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};

const PREFIX: &str =
    "program T; type Point = record X: integer; end; begin var P: Point := record X := 1; end;";

#[test]
fn record_update_rejects_non_record_base() {
    let errors = check_errors(
        "program T; begin var X: integer := 1; var Y: integer := X with Value := 2; end end.",
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "{errors:#?}"
    );
}

#[test]
fn record_update_rejects_unknown_and_wrongly_typed_fields() {
    for (update, expected_code) in [
        ("P with Missing := 2; end", SEMA_UNKNOWN_NAME),
        ("P with X := 'wrong'; end", SEMA_TYPE_MISMATCH),
    ] {
        let source = format!("{PREFIX} var Q: Point := {update} end.");
        let errors = check_errors(&source);
        assert!(
            errors.iter().any(|error| error.code == expected_code),
            "update: {update}; errors: {errors:#?}"
        );
    }
}

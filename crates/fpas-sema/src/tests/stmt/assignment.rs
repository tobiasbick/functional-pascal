use super::super::{check_errors, check_ok};

#[test]
fn assign_mutable() {
    check_ok(
        "program T; \
         mutable var X: integer := 0; \
         begin X := 1 end.",
    );
}

#[test]
fn assign_immutable_error() {
    check_errors(
        "program T; \
         var X: integer := 0; \
         begin X := 1 end.",
    );
}

#[test]
fn assign_type_mismatch() {
    check_errors(
        "program T; \
         mutable var X: integer := 0; \
         begin X := true end.",
    );
}

#[test]
fn assign_undefined_error() {
    check_errors("program T; begin Y := 1 end.");
}

#[test]
fn member_assign_undefined_receiver_reports_once() {
    let errors = check_errors("program T; begin B.OnClick := 1 end.");
    let unknown = errors
        .iter()
        .filter(|error| error.code == fpas_diagnostics::codes::SEMA_UNKNOWN_NAME)
        .count();
    assert_eq!(
        unknown, 1,
        "expected a single undefined-identifier diagnostic, got: {errors:#?}"
    );
}

#[test]
fn assign_to_array_element_ok() {
    check_ok(
        "program T; \
         begin \
         mutable var A: array of integer := [1, 2, 3]; \
         A[0] := 99 \
         end.",
    );
}

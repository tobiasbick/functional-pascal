//! Regression coverage for strict generic-body compatibility.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

#[test]
fn generic_body_rejects_concrete_return_for_type_parameter() {
    let errors = check_errors(
        "program T;
         function F<T>(X: T): T;
         begin return 'hello' end;
         begin end.",
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "expected type mismatch, got {errors:#?}"
    );
}

#[test]
fn generic_body_rejects_type_parameter_as_boolean_condition() {
    let errors = check_errors(
        "program T;
         function F<T>(X: T): T;
         begin
           if X then return X;
           return X
         end;
         begin end.",
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "expected type mismatch, got {errors:#?}"
    );
}

#[test]
fn generic_body_accepts_same_type_parameter_and_call_site_inference() {
    check_ok(
        "program T;
         function Identity<T>(X: T): T;
         begin return X end;
         var Value: integer := Identity(42);
         begin end.",
    );
}

#[test]
fn generic_function_value_does_not_coerce_to_concrete_signature() {
    let errors = check_errors(
        "program T;
         function Identity<T>(X: T): T;
         begin return X end;
         var Concrete: function(X: integer): string := Identity;
         begin end.",
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "expected type mismatch, got {errors:#?}"
    );
}

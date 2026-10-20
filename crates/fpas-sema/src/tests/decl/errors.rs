//! Static native-error boundaries; execution and artifacts have separate owner tests.
//! Documentation: `docs/pascal/language/types/errors.md`.

use super::check_errors;
use fpas_diagnostics::codes::{SEMA_CONSTRAINT_VIOLATION, SEMA_TYPE_MISMATCH};

#[test]
fn error_payloads_require_printable_data_and_a_typed_cause() {
    for (parameters, field) in [
        ("", "Cause: string"),
        ("", "Cause: Option of integer"),
        (" of E", "Cause: E"),
        (" of E: Printable", "Cause: E"),
        (" of T", "Data: T"),
        ("", "Callback: function(): integer"),
        ("", "Value: Token"),
    ] {
        let source = format!(
            "program T; type Token = distinct integer; type Failure{parameters} = error Failed({field}); end error; begin end."
        );
        let errors = check_errors(&source);
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
                && error
                    .span
                    .is_some_and(|span| source[span.offset()..span.end()].contains(field))),
            "{field}: {errors:#?}"
        );
    }
}

#[test]
fn error_constraint_never_accepts_strings_or_ordinary_enums() {
    for (declaration, value) in [
        ("", "'text'"),
        ("type Status = enum Failed; end enum;", "Status.Failed"),
    ] {
        let source = format!(
            "program T; {declaration} function Describe<E: Error>(Value: E): string; begin return Value.Message(); end function; begin discard Describe({value}); end."
        );
        let errors = check_errors(&source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
            "{errors:#?}"
        );
    }
}

#[test]
fn native_errors_preserve_nominal_assignment_and_reject_ordering() {
    for body in [
        "const Value: FirstError := SecondError.Failed;",
        "const Less := FirstError.Failed < FirstError.Failed;",
    ] {
        let source = format!(
            "program T; type FirstError = error Failed; end error; type SecondError = error Failed; end error; begin {body} end."
        );
        check_errors(&source);
    }
}

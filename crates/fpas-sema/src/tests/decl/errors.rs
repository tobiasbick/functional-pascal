//! Static native-error boundaries; execution and artifacts have separate owner tests.
//! Documentation: `docs/pascal/language/types/errors.md`.

use super::{check_errors, check_ok};
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

#[test]
fn constructor_values_reject_incompatible_signatures_and_missing_type_arguments() {
    for body in [
        "const F: function(): Failure := Failure.Bad;",
        "const F: function(Value: integer): Failure := Failure.Bad;",
        "const F: function(var Text: string): Failure := Failure.Bad;",
        "const F := Phantom.Bad;",
        "const F: function(Text: string): Failure := Failure.Bad; discard F(Value := 'bad');",
        "const F: function(): Failure := Failure.Empty;",
        "discard Failure.Empty();",
        "const R: Result of (integer, string) := Error('bad'); discard R.MapError(Failure.Pair);",
        "const R: Result of (integer, string) := Error('bad'); discard R.MapError(Failure.Bad).Map(function(V: string): string begin return V; end function);",
    ] {
        let source = format!(
            "program T; type Failure = error Bad(Text: string); Pair(Text: string; Code: integer); Empty; end error; type Phantom of T: Printable = error Bad(Text: string); end error; begin {body} end."
        );
        assert!(
            !check_errors(&source).is_empty(),
            "accepted incompatible constructor: {body}"
        );
    }
}

#[test]
fn try_requires_explicit_mapping_between_distinct_error_families() {
    let declarations = "type First = error Bad(Text: string); end error; type Second = error Bad(Value: First); end error;";
    for (expression, accepted) in [("Value", false), ("Value.MapError(Second.Bad)", true)] {
        let source = format!(
            "program T; {declarations} function Work(Value: Result of (integer, First)): Result of (integer, Second); begin const N := try {expression}; return Ok(N); end function; begin end."
        );
        if accepted {
            check_ok(&source);
        } else {
            check_errors(&source);
        }
    }
}

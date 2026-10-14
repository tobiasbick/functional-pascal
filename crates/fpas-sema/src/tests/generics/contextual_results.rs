//! Expected results complete generic arguments without replacing supplied evidence.
//! Documentation: docs/pascal/language/functions/generic-routines.md.

use crate::tests::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_CONSTRAINT_VIOLATION, SEMA_TYPE_MISMATCH};

const ROUTINES: &str = "
function Empty<T>(): Option of T; begin return None; end function;
function Identity<T>(Value: T): T; begin return Value; end function;
function NumericValue<T: Numeric>(): Option of T; begin return None; end function;
function Unused<T>(): integer; begin return 1; end function;";

#[test]
fn missing_generic_arguments_are_rejected_before_lowering() {
    for statement in [
        "const Value := Empty();",
        "discard Empty();",
        "discard Unused();",
        "const Value := Identity(Unused());",
        "const Factory: function(): integer := Unused; const Value := Factory();",
    ] {
        let errors = check_errors(&format!("program T; {ROUTINES} begin {statement} end."));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
                && error.message.contains("Cannot infer")
                && error
                    .help
                    .as_deref()
                    .is_some_and(|hint| hint.contains("annotation"))),
            "{statement}: {errors:?}"
        );
    }
}

#[test]
fn expected_types_do_not_override_arguments_or_bypass_constraints() {
    for statement in [
        "const Value: Option of real := Identity(Some(1));",
        "const Value: real := Identity(1);",
        "const Value: Result of (real, string) := Identity(Ok(1));",
    ] {
        let errors = check_errors(&format!("program T; {ROUTINES} begin {statement} end."));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{errors:?}"
        );
    }
    let errors = check_errors(&format!(
        "program T; {ROUTINES} begin const Value: Option of string := NumericValue(); end."
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
        "{errors:?}"
    );
    check_ok(&format!(
        "program T; {ROUTINES} begin const Value: Option of integer := NumericValue(); end."
    ));
}

#[test]
fn native_callbacks_require_complete_generic_arguments() {
    let errors = check_errors(
        "program T; function Empty<T>(Value: integer): Option of T; begin return None; end function; begin const Values := [1].Map(Empty); end.",
    );
    assert!(errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH && error.message.contains("Cannot infer")), "{errors:?}");
}

#[test]
fn deeply_nested_repeated_routine_names_keep_distinct_parameters() {
    let errors = check_errors(
        "program T;
        function Outer<X>(Value: X): integer;
        function A<T>(Other: T): integer;
        function A<U>(Saved: U): integer;
        function A<U>(Input: U): U; begin return Saved; end function;
        begin const Wrong: integer := A(1); return Wrong + 1; end function;
        begin return A('bad'); end function;
        begin return A(Value); end function;
        begin const Value := Outer(1); end.",
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
            && error
                .help
                .as_deref()
                .is_some_and(|hint| hint.contains("different type parameter declarations"))),
        "{errors:?}"
    );
}

#[test]
fn complete_outer_types_cannot_hide_unresolved_inner_calls_or_constructors() {
    for initializer in [
        "if (Empty() is None) then 1 else 2 end if",
        "case Empty() of when None: 1; when Some(_): 2; end case",
        "[function(): integer begin discard Empty(); return 1; end function]",
        "[(Empty() is None)]",
        "Identity(Empty() is None)",
        "[Defaults().Ready()]",
        "[Holder(Value := Identity).Ready()]",
        "[(Some(Identity) is Some(_))]",
    ] {
        let errors = check_errors(&format!(
            "program T; {ROUTINES}
            type Defaults of T = record Value: Option of T := None;
                function Ready(Self: Defaults of T): boolean; begin return true; end function;
            end record;
            type Holder of T = record Value: T;
                function Ready(Self: Holder of T): boolean; begin return true; end function;
            end record;
            begin const Value := {initializer}; end."
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TYPE_MISMATCH
                    && error.message.contains("Cannot infer")),
            "{initializer}: {errors:?}"
        );
    }
}

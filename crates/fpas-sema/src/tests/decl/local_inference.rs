//! Literal evidence, resolved conversions, and explicit-annotation boundaries.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_TYPE_ANNOTATION_REQUIRED, SEMA_TYPE_MISMATCH};

const PREFIX: &str = "program T; uses Std.Conv; type UserId = distinct integer; type Raw = integer;
type Point = record X: integer; end record;
type Shape = enum Circle(Radius: real); end enum;
function ReadValue(): integer; begin return 42; end function;";

#[test]
fn local_literals_and_conversions_have_fixed_types() {
    check_ok(&format!(
        "{PREFIX} begin
        const Whole := 42; const Signed := (-2); const LowerBound := -9223372036854775807;
        const Fraction := 1.25; const Text := 'Ada'; const Flag := true;
        var Count := 1; Count := Count + Whole;
        const Numbers := [Whole, ReadValue()]; const Matrix := [[1], [2]];
        const Index := ['a': 1, 'b': 2]; const Nested := ['a': [1], 'b': [2]];
        const Id := (UserId(Whole)); const Unwrapped := Raw(Id);
        const Keep: UserId := Id; const Value: integer := Unwrapped;
        const Copy: array of integer := Numbers; const Lookup: dict of string to integer := Index;
        end."
    ));
}

#[test]
fn other_initializer_forms_require_an_annotation() {
    for initializer in [
        "ReadValue()",
        "'x'.ToUpper()",
        "IntToReal(1)",
        "1 + 2",
        "not true",
        "Whole",
        "if true then 1 else 2 end if",
        "None",
        "Ok(1)",
        "function(): integer begin return 1; end function",
    ] {
        let source =
            format!("{PREFIX} begin const Whole: integer := 1; const Value := {initializer}; end.");
        let errors = check_errors(&source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TYPE_ANNOTATION_REQUIRED),
            "{initializer}: {errors:#?}"
        );
    }
}

#[test]
fn empty_or_inconsistent_collections_require_an_annotation() {
    for initializer in [
        "[]",
        "[:]",
        "[1, 2.0]",
        "[1.0, 2]",
        "[[1], []]",
        "[[], [1]]",
        "['a': []]",
        "['a': [1], 'b': []]",
        "['a': 1, 'b': 2.0]",
        "[1: 'a', 2.0: 'b']",
        "[None]",
        "[Ok(1)]",
    ] {
        let errors = check_errors(&format!("{PREFIX} begin var Value := {initializer}; end."));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TYPE_ANNOTATION_REQUIRED
                    && error
                        .help
                        .as_deref()
                        .is_some_and(|hint| hint.contains("var Value: Type :="))),
            "{initializer}: {errors:#?}"
        );
    }
}

#[test]
fn explicit_annotations_continue_to_supply_collection_context() {
    check_ok(
        "program T; begin const Empty: array of integer := []; const D: dict of string to integer := [:]; const Nested: array of array of integer := [[1], []]; const Options: array of Option of integer := [None]; end.",
    );
}

#[test]
fn inferred_variables_reject_type_changes_and_constants_reject_reassignment() {
    let errors = check_errors("program T; begin var Number := 1; Number := 'wrong'; end.");
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "{errors:#?}"
    );
    check_errors("program T; begin const Number := 1; Number := 2; end.");
}

#[test]
fn a_callable_shadowing_a_type_does_not_count_as_a_conversion() {
    let errors = check_errors(
        "program T; type UserId = distinct integer; begin const UserId: function(Value: integer): integer := function(Value: integer): integer begin return Value; end function; const Id := UserId(1); end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_TYPE_ANNOTATION_REQUIRED),
        "{errors:#?}"
    );
}

#[test]
fn named_nested_routines_capture_inferred_local_types() {
    check_ok(
        "program T; function Outer(): integer; function Inner(): integer; begin return Saved + Count; end function; begin const Saved := 7; var Count := 2; return Inner(); end function; begin const Value: integer := Outer(); end.",
    );
}

#[test]
fn invalid_conversions_keep_the_original_diagnostic() {
    let errors = check_errors(&format!("{PREFIX} begin const Id := UserId('wrong'); end."));
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "{errors:#?}"
    );
    assert!(
        !errors
            .iter()
            .any(|error| error.code == SEMA_TYPE_ANNOTATION_REQUIRED),
        "{errors:#?}"
    );
}

#[test]
fn inferred_aggregates_preserve_task_and_callable_discard_restrictions() {
    for initializer in [
        "[Job]",
        "[function(): integer begin return Wait(Job); end function]",
        "Some(Job)",
        "Holder(Value := Job)",
        "Envelope.Held(Job)",
    ] {
        let errors = check_errors(&format!(
            "program T; uses Std.Tasks; type Holder = record Value: task of integer; end record; type Envelope of T = enum Held(Value: T); end enum; function Work(): integer; begin return 1; end function; begin const Job: task of integer := go Work(); const Values := {initializer}; discard Values; end."
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == fpas_diagnostics::codes::SEMA_UNSAFE_DISCARD),
            "{initializer}: {errors:#?}"
        );
        assert!(
            !errors
                .iter()
                .any(|error| error.code == SEMA_TYPE_ANNOTATION_REQUIRED),
            "{initializer}: {errors:#?}"
        );
    }
}

#[test]
fn generic_callable_collection_elements_require_an_annotation() {
    let errors = check_errors(
        "program T; function Identity<T>(Value: T): T; begin return Value; end function; begin const Values := [Identity]; end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_TYPE_ANNOTATION_REQUIRED),
        "{errors:#?}"
    );
}

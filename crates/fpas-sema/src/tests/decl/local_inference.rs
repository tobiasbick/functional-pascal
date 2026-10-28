//! Fully determined local expressions and explicit-annotation boundaries.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_TYPE_ANNOTATION_REQUIRED, SEMA_TYPE_MISMATCH};

const PREFIX: &str = "program T; uses Std.Math as Math; type UserId = distinct integer; type Raw = integer;\ntype Point = record X: integer; end record;\ntype Shape = enum Circle(Radius: real); end enum;\nfunction ReadValue(): integer; begin return 42; end function;";

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
fn all_fully_determined_initializer_forms_infer() {
    for initializer in [
        "ReadValue()",
        "'x'.ToUpper()",
        "Math.IntToReal(1)",
        "1 + 2",
        "not true",
        "Whole",
        "if true then 1 else 2 end if",
        "function(): integer begin return 1; end function",
    ] {
        let source =
            format!("{PREFIX} begin const Whole: integer := 1; const Value := {initializer}; end.");
        check_ok(&source);
    }
}

#[test]
fn underdetermined_collections_require_an_annotation() {
    for initializer in ["[]", "[:]", "['a': []]", "[None]", "[Ok(1)]"] {
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
fn collection_evidence_completes_missing_elements_without_numeric_promotion() {
    for initializer in [
        "[[1], []]",
        "[[], [1]]",
        "['a': [1], 'b': []]",
        "[None, Some(1)]",
        "[Some(1), None]",
        "[Ok(1), Error('failed')]",
    ] {
        check_ok(&format!(
            "{PREFIX} begin const Value := {initializer}; end."
        ));
    }
    for initializer in [
        "[1, 2.0]",
        "[1.0, 2]",
        "['a': 1, 'b': 2.0]",
        "[1: 'a', 2.0: 'b']",
    ] {
        let errors = check_errors(&format!(
            "{PREFIX} begin const Value := {initializer}; end."
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
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
fn a_callable_shadowing_a_type_keeps_its_ordinary_result_type() {
    check_ok(
        "program T; type UserId = distinct integer; begin const UserId: function(Value: integer): integer := function(Value: integer): integer begin return Value; end function; const Id := UserId(1); end.",
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
fn inferred_aggregates_allow_discard_without_losing_task_escape_checks() {
    for initializer in [
        "[Job]",
        "Some(Job)",
        "Holder(Value := Job)",
        "Envelope.Held(Job)",
    ] {
        check_ok(&format!(
            "program T; uses Std.Tasks as Tasks; type Holder = record Value: task of integer; end record; type Envelope of T = enum Held(Value: T); end enum; function Work(): integer; begin return 1; end function; begin scope  const Job: task of integer := go Work(); const Values := {initializer}; discard Values;  end scope; end."
        ));
    }
}

#[test]
fn generic_callable_collection_elements_require_an_annotation() {
    let errors = check_errors(
        "program T; function Identity<T>(Value: T): T; begin return Value; end function; begin const Values := [Identity]; end.",
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
            && error
                .help
                .as_deref()
                .is_some_and(|hint| hint.contains("callable signature"))),
        "{errors:#?}"
    );
}

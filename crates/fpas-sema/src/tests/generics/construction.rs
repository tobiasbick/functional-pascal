//! Named generic record construction and contextual arguments.

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{
    SEMA_CONSTRAINT_VIOLATION, SEMA_MISSING_RECORD_FIELD, SEMA_TYPE_MISMATCH,
};

#[test]
fn constructors_infer_supplied_fields_and_use_expected_arguments() {
    check_ok(
        r#"program Main;
        type Pair of (K, V) = record Key: K; Value: V; Count: integer := 3; end record;
        type Marker of (T) = record Count: integer := 3; end record;
        type IntegerPair = Pair of (integer, string);
        function Make(): Pair of (integer, string); begin return Pair(Key := 1, Value := 'one'); end function;
        function MakeMarker(): Marker of (integer); begin return Marker(); end function;
        procedure Accept(Item: Pair of (integer, string)); begin null; end procedure;
        const Item: IntegerPair := IntegerPair(Key := 1, Value := 'one');
        begin Accept(Pair(Value := 'one', Key := 1)); discard MakeMarker(); end program;"#,
    );
}

#[test]
fn constructors_forward_enclosing_generic_parameters() {
    check_ok(
        "program Main;
        type Box of (T: Equatable) = record Value: T; end record;
        function Make of (U: Equatable)(Value: U): Box of (U);
        begin return Box(Value := Value); end function;
        begin discard Make(42); end program;",
    );
}

#[test]
fn enum_constructors_infer_payloads_and_contextual_missing_arguments() {
    check_ok(
        r#"program Main;
        type Lookup of (T, E) = enum Found(Value: T); Missing; Failed(Message: E); end enum;
        function Make of (U)(Value: U): Lookup of (U, string); begin return Lookup.Found(Value); end function;
        const Present: Lookup of (integer, string) := Lookup.Found(42);
        const Absent: Lookup of (integer, string) := Lookup.Missing;
        const FailureValue: Lookup of (integer, string) := Lookup.Failed('failure');
        begin discard Make(42); end program;"#,
    );
    check_ok(
        r#"program Main;
        type Choice of (T) = enum Present(Value: T); Missing; end enum;
        type Box of (T) = record Value: T; end record;
        const Converted: Choice of (real) := Choice.Present(42.0);
        const RecordValue: Box of (real) := Box(Value := 42.0);
        begin discard Choice.Present('inferred'); end program;"#,
    );
}

#[test]
fn constructor_arguments_infer_generic_routine_parameters() {
    check_ok(
        "program Main;
        type Box of (T) = record Value: T; end record;
        function Unbox of (U)(Item: Box of (U)): U; begin return Item.Value; end function;
        begin discard Unbox(Box(Value := 42)); end program;",
    );
}

#[test]
fn enum_construction_requires_complete_arguments_and_positional_payloads() {
    for (expression, message) in [
        ("Choice.Missing", "Cannot infer every type argument"),
        ("Choice.Missing()", "Payload-less variant"),
        ("Choice.Present", "expects 1 argument"),
        ("Choice.Present(1, 2)", "expects 1 argument"),
    ] {
        let errors = check_errors(&format!(
            "program Main;
            type Choice of (T) = enum Present(Value: T); Missing; end enum;
            begin discard {expression}; end program;"
        ));
        assert!(
            errors.iter().any(|error| error.message.contains(message)),
            "{expression}: {errors:#?}"
        );
    }
    let errors = check_errors(
        "program Main;
        type Choice of (T: Numeric) = enum Present(Value: T); Missing; end enum;
        begin discard Choice.Present('wrong'); end program;",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
        "{errors:#?}"
    );
    let errors = check_errors(
        r#"program Main;
        type Choice of (T) = enum Present(Value: T); Missing; end enum;
        const Item: Choice of (real) := Choice.Present(42);
        begin null; end program;"#,
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "{errors:#?}"
    );
}

#[test]
fn unresolved_constructor_arguments_request_an_annotation() {
    for source in [
        "type Box of (T) = record end record; begin discard Box();",
        "type Box of (T) = record Value: T; end record; begin discard Box(Value := Option.None);",
    ] {
        let errors = check_errors(&format!("program Main; {source} end program;"));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("Cannot infer every type argument")),
            "{errors:#?}"
        );
    }
}

#[test]
fn contextual_fields_and_empty_components_constrain_constructor_arguments() {
    check_ok(
        r#"program Main;
        type Box of (T) = record Value: T; end record;
        type Pair of (T) = record First: T; Second: T; end record;
        const Item: Box of (Option of (integer)) := Box(Value := Option.None);
        begin discard Pair(First := Option.None, Second := Option.Some(42)); end program;"#,
    );
}

#[test]
fn constructors_check_constraints_and_inconsistent_field_bindings() {
    let errors = check_errors(
        "program Main;
        type Box of (T: Numeric) = record Value: T; end record;
        begin discard Box(Value := 'wrong'); end program;",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
        "{errors:#?}"
    );
    let errors = check_errors(
        "program Main;
        type Pair of (T) = record First: T; Second: T; end record;
        begin discard Pair(First := 1, Second := 'wrong'); end program;",
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "{errors:#?}"
    );
}

#[test]
fn construction_rejects_missing_unknown_duplicate_and_positional_fields() {
    for (constructor, message) in [
        ("Pair(First := 1)", "Required field"),
        ("Pair(First := 1, Missing := 2)", "has no field"),
        (
            "Pair(First := 1, first := 2, Second := 3)",
            "specified more than once",
        ),
        ("Pair(1, 2)", "requires named fields"),
    ] {
        let errors = check_errors(&format!(
            "program Main;
            type Pair = record First: integer; Second: integer; end record;
            begin discard {constructor}; end program;"
        ));
        assert!(
            errors.iter().any(|error| error.message.contains(message)),
            "{constructor}: {errors:#?}"
        );
        if message == "Required field" {
            assert!(
                errors
                    .iter()
                    .any(|error| error.code == SEMA_MISSING_RECORD_FIELD)
            );
        }
    }
}

#[test]
fn named_arguments_on_routines_and_variants_are_errors() {
    for (declarations, target) in [
        (
            "function Make(Value: integer): integer; begin return Value; end function;",
            "Make",
        ),
        (
            "type Choice = enum Make(Value: integer); Missing; end enum;",
            "Choice.Make",
        ),
    ] {
        let errors = check_errors(&format!(
            "program Main; {declarations} begin discard {target}(Value := 1); end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("Named fields require a record type")),
            "{errors:#?}"
        );
    }
}

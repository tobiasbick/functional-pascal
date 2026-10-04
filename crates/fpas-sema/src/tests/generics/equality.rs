//! Equality-only generic constraints and their forwarding rules.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_CONSTRAINT_VIOLATION;

#[test]
fn equatable_accepts_nested_value_data_and_recursive_types() {
    check_ok(
        "program Main;\n        type Tree = enum Leaf(Value: integer); Children(Values: array of (Tree)); end enum;\n        function Same of (T: Equatable)(A: T; B: T): boolean;\n        begin return A = B; end function;\n        begin\n          discard Same([1, 2], [1, 2]);\n          discard Same(['a': [1]], ['a': [1]]);\n          discard Same(Tree.Leaf(1), Tree.Children([]));\n        end program;",
    );
}

#[test]
fn equatable_does_not_permit_ordering_or_arithmetic() {
    for operation in ["A < B", "A + B = A"] {
        let errors = check_errors(&format!(
            "program Main;\n            function Compare of (T: Equatable)(A: T; B: T): boolean;\n            begin return {operation}; end function;\n            begin null; end program;"
        ));
        assert!(!errors.is_empty(), "{operation}");
    }
}

#[test]
fn equatable_forwarding_checks_capabilities_independently_of_names() {
    for constraint in ["Equatable", "Comparable", "Numeric"] {
        check_ok(&format!(
            "program Main;\n            function Same of (T: Equatable)(A: T; B: T): boolean;\n            begin return A = B; end function;\n            function Forward of (U: {constraint})(Value: U): boolean;\n            begin return Same(Value, Value); end function;\n            begin null; end program;"
        ));
    }
    for constraint in ["", ": Printable"] {
        let errors = check_errors(&format!(
            "program Main;\n            function Same of (T: Equatable)(A: T; B: T): boolean;\n            begin return A = B; end function;\n            function Forward of (T{constraint})(Value: T): boolean;\n            begin return Same(Value, Value); end function;\n            begin null; end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
            "{constraint}: {errors:#?}"
        );
    }
}

#[test]
fn equatable_rejects_nested_callable_types() {
    let errors = check_errors(
        r#"program Main;
        function Same of (T: Equatable)(A: T; B: T): boolean;
        begin return A = B; end function;
        begin
          const Values: array of (option of (procedure())) := [Option.None];
          discard Same(Values, Values);
        end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
        "{errors:#?}"
    );
}

#[test]
fn empty_constructors_receive_equality_context_from_all_arguments() {
    check_ok(
        "program Main;\n         function Same of (T: Equatable)(A: T; B: T): boolean;\n         begin return A = B; end function;\n         begin\n           discard Same(Option.None, Option.Some(1)); discard Same(Option.Some(1), Option.None);\n           discard Same(Result.Ok(1), Result.Error('failed')); discard Same(Result.Error('failed'), Result.Ok(1));\n           discard Same([], [[Option.None, Option.Some(1)]]);\n           discard Same([[Option.None]], [[Option.Some(1)]]);\n           discard Same(['a': Option.None], ['a': Option.Some(1)]);\n         end program;",
    );
}

#[test]
fn empty_arguments_cannot_hide_callable_components_from_constraints() {
    for (left, right) in [
        ("[]", "[Handler]"),
        ("[Option.None]", "[Option.Some(Handler)]"),
        ("Option.None", "Option.Some(Handler)"),
        ("Result.Ok(1)", "Result.Error(Handler)"),
        ("[[]]", "[[Handler]]"),
        ("['a': Option.None]", "['a': Option.Some(Handler)]"),
        (
            "Box(Value := Option.None)",
            "Box(Value := Option.Some(Handler))",
        ),
    ] {
        for (left, right) in [(left, right), (right, left)] {
            let errors = check_errors(&format!(
                "program Main;\n                 type Box of (T) = record Value: T; end record;\n                 procedure Handler(); begin null; end procedure;\n                 function Same of (T: Equatable)(A: T; B: T): boolean;\n                 begin return A = B; end function;\n                 begin discard Same({left}, {right}); end program;"
            ));
            assert!(
                errors
                    .iter()
                    .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
                "{left}, {right}: {errors:#?}"
            );
        }
    }
}

#[test]
fn empty_arguments_cannot_hide_resource_components_from_constraints() {
    for call in [
        "Same([Option.None], [Handle])",
        "Same([Handle], [Option.None])",
    ] {
        let errors = check_errors(&format!(
            r#"program Main; uses Std.Net as Net;
             function Same of (T: Equatable)(A: T; B: T): boolean;
             begin return A = B; end function;
             begin const Handle: option of (Net.Connection) := Option.None;
               discard {call};
             end program;"#
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
            "{call}: {errors:#?}"
        );
    }
}

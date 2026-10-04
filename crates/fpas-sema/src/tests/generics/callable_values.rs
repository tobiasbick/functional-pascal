//! Expected callable signatures instantiate only the selected declaration's parameters.
//!
//! **Documentation:** `docs/pascal/language/functions/generic-routines.md`.

use crate::tests::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_CONSTRAINT_VIOLATION, SEMA_TYPE_MISMATCH};

#[test]
fn generic_callable_values_use_explicit_and_argument_contexts() {
    check_ok(
        "program Main;
        function Identity of (T)(Value: T): T; begin return Value; end function;
        procedure Ignore of (T)(Value: T); begin discard Value; end procedure;
        function Apply of (T)(Value: T; Callback: function(Input: T): T): T;
        begin return Callback(Value); end function;
        function Before of (T)(Callback: function(Input: T): T; Value: T): T;
        begin return Callback(Value); end function;
        function Forward of (T)(Value: T): T; begin return Apply(Value, Identity); end function;
        begin
          const Callback: function(Input: integer): integer := Identity;
          const Action: procedure(Input: string) := Ignore;
          const Functions: array of (function(Input: integer): integer) := [Identity];
          const Answer: integer := Before(Identity, Apply(42, Identity));
          const Text: string := Forward('text');
          Action(Text); discard Callback(Answer); discard Functions[0](Answer);
        end program;",
    );
}

#[test]
fn generic_callable_values_preserve_constraints_and_repeated_parameters() {
    for (constraint, annotation) in [
        ("Numeric", "function(Value: string): string"),
        (
            "Equatable",
            "function(Value: function(): integer): function(): integer",
        ),
    ] {
        let errors = check_errors(&format!(
            "program Main;
            function Identity of (T: {constraint})(Value: T): T; begin return Value; end function;
            begin const Callback: {annotation} := Identity; end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
            "{errors:#?}"
        );
    }
    let errors = check_errors(
        "program Main;
        function First of (T)(Left: T; Right: T): T; begin return Left; end function;
        begin const Callback: function(Left: integer; Right: string): integer := First; end program;",
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "{errors:#?}"
    );
}

#[test]
fn generic_callable_values_require_concrete_context_inside_containers() {
    for initializer in [
        "[Identity]",
        "[[Identity]]",
        "['one': Identity]",
        "Option.Some(Identity)",
        "Box(Value := Identity)",
        "Choice.Present(Identity)",
        "if true then [Identity] else [Identity] end if",
    ] {
        let errors = check_errors(&format!(
            "program Main;
            type Box of (T) = record Value: T; end record;
            type Choice of (T) = enum Present(Value: T); Missing; end enum;
            function Identity of (T)(Value: T): T; begin return Value; end function;
            begin const Value := {initializer}; end program;"
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{initializer}: {errors:#?}"
        );
    }
    check_ok(
        "program Main;
        type Handler = function(Value: integer): integer;
        type Box of (T) = record Value: T; end record;
        function Identity of (T)(Value: T): T; begin return Value; end function;
        begin
          const Arrays: array of (array of (Handler)) := [[Identity]];
          const Dictionary: dict of (string, Handler) := ['one': Identity];
          const Optional: Option of (Handler) := Option.Some(Identity);
          const Success: Result of (Handler, string) := Result.Ok(Identity);
          const Boxed: Box of (Handler) := Box(Value := Identity);
          discard Arrays; discard Dictionary; discard Optional; discard Success; discard Boxed;
        end program;",
    );
}

#[test]
fn generic_routine_arguments_must_determine_every_concrete_parameter() {
    for call in ["Ignore(Identity)", "Unused()"] {
        let errors = check_errors(&format!(
            "program Main;
            function Identity of (T)(Value: T): T; begin return Value; end function;
            function Ignore of (T)(Value: T): integer; begin return 1; end function;
            function Unused of (T)(): integer; begin return 1; end function;
            begin discard {call}; end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TYPE_MISMATCH
                    && error.message.contains("Cannot infer")),
            "{call}: {errors:#?}"
        );
    }
}

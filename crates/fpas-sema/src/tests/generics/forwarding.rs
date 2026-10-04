//! Generic arguments retain their caller's type and constraint.
//!
//! **Documentation:** `docs/pascal/language/functions/generic-routines.md`.

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_CONSTRAINT_VIOLATION, SEMA_TYPE_MISMATCH};

#[test]
fn numeric_forwarding_accepts_independent_parameter_names() {
    for caller in ["T", "U", "t"] {
        check_ok(&format!(
            "program Main;\n             function Twice of (T: Numeric)(Value: T): T;\n             begin return Value + Value; end function;\n             function Forward of ({caller}: Numeric)(Value: {caller}): {caller};\n             begin return Twice(Value); end function;\n             begin discard Forward(21); end program;"
        ));
    }
}

#[test]
fn unconstrained_forwarding_cannot_bypass_numeric() {
    for caller in ["T", "U"] {
        let errors = check_errors(&format!(
            "program Main;\n             function Twice of (T: Numeric)(Value: T): T;\n             begin return Value + Value; end function;\n             function Forward of ({caller})(Value: {caller}): {caller};\n             begin return Twice(Value); end function;\n             begin discard Forward('x'); end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
            "expected constraint violation for {caller}, got {errors:#?}"
        );
    }
}

#[test]
fn weaker_constraints_cannot_be_forwarded_to_numeric() {
    for constraint in ["Comparable", "Printable"] {
        let errors = check_errors(&format!(
            "program Main;\n             function Twice of (T: Numeric)(Value: T): T;\n             begin return Value + Value; end function;\n             function Forward of (U: {constraint})(Value: U): U;\n             begin return Twice(Value); end function;\n             begin null; end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
            "expected constraint violation for {constraint}, got {errors:#?}"
        );
    }
}

#[test]
fn numeric_forwarding_satisfies_comparable_and_printable() {
    check_ok(
        "program Main; uses Std.Console as Console;\n         function Less of (T: Comparable)(First: T; Second: T): boolean;\n         begin return First < Second; end function;\n         procedure Print of (T: Printable)(Value: T);\n         begin Console.WriteLn(Value); end procedure;\n         procedure Forward of (U: Numeric)(Value: U);\n         begin discard Less(Value, Value); Print(Value); end procedure;\n         begin Forward(21); end program;",
    );
}

#[test]
fn forwarding_infers_nested_array_and_callable_parameters() {
    check_ok(
        "program Main;\n         function Apply of (T: Numeric)(Values: array of (T); Action: function(Item: T): T): T;\n         begin return Action(Values[0]); end function;\n         function Forward of (U: Numeric)(Values: array of (U); Action: function(Input: U): U): U;\n         begin return Apply(Values, Action); end function;\n         begin null; end program;",
    );
}

#[test]
fn procedures_validate_forwarded_constraints() {
    check_ok(
        "program Main;\n         procedure Consume of (T: Numeric)(Value: T); begin discard Value; end procedure;\n         procedure Forward of (U: Numeric)(Value: U); begin Consume(Value); end procedure;\n         begin Forward(42); end program;",
    );
    let errors = check_errors(
        "program Main;\n         procedure Consume of (T: Numeric)(Value: T); begin discard Value; end procedure;\n         procedure Forward of (T)(Value: T); begin Consume(Value); end procedure;\n         begin Forward('x'); end program;",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION)
    );
}

#[test]
fn unrelated_caller_type_parameters_do_not_unify() {
    let errors = check_errors(
        "program Main;\n         function First of (T)(Left: T; Right: T): T; begin return Left; end function;\n         function Forward of (U, V)(Left: U; Right: V): U;\n         begin return First(Left, Right); end function;\n         begin null; end program;",
    );
    assert!(errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH));
}

#[test]
fn unconstrained_identity_forwarding_preserves_the_caller_type() {
    check_ok(
        "program Main;\n         function Identity of (T)(Value: T): T; begin return Value; end function;\n         function Forward of (U)(Value: U): U; begin return Identity(Value); end function;\n         begin var Text: string := Forward('x'); end program;",
    );
}

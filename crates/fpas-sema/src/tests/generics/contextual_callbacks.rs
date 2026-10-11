//! Joint inference and contextual specialization of callbacks inside containers.

use crate::tests::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_CONSTRAINT_VIOLATION, SEMA_TYPE_MISMATCH};

const ROUTINES: &str = "
function Identity<U>(Value: U): U; begin return Value; end function;
procedure Ignore<U>(Value: U); begin end procedure;
function Number<U: Numeric>(Value: U): U; begin return Value; end function;
function Text(Value: integer): string; begin return 'text'; end function;";

#[test]
fn constructor_siblings_specialize_nested_callbacks_in_either_order() {
    for (callback_type, callbacks) in [
        ("array of function(Value: T): T", "[Identity]"),
        ("array of procedure(Value: T)", "[Ignore]"),
        (
            "Option of array of function(Value: T): T",
            "Some([Identity])",
        ),
        (
            "dict of string to function(Value: T): T",
            "['id': Identity]",
        ),
        (
            "Result of (array of function(Value: T): T, string)",
            "Ok([Identity])",
        ),
    ] {
        let declarations = format!(
            "type Handler of T = record Callbacks: {callback_type}; Value: T; end record;
             type Choice of T = enum Ready(Callbacks: {callback_type}; Value: T); end enum;"
        );
        for fields in [
            format!("Callbacks := {callbacks}, Value := 3"),
            format!("Value := 3, Callbacks := {callbacks}"),
        ] {
            for constructor in ["Handler", "Choice.Ready"] {
                check_ok(&format!(
                    "program T; {declarations} {ROUTINES} begin const H := {constructor}({fields}); end."
                ));
            }
        }
    }
}

#[test]
fn routine_arguments_and_enclosing_parameters_supply_nested_callback_context() {
    check_ok(&format!(
        "program T; {ROUTINES}
         type Handler of T = record Callbacks: array of function(Value: T): T; Value: T; end record;
         function Make<T>(Value: T): Handler of T;
         begin return Handler(Callbacks := [Identity], Value := Value); end function;
         procedure Accept<T>(Callbacks: array of function(Value: T): T; Evidence: T); begin end procedure;
         begin Accept([Identity], 3); Accept(Evidence := 3, Callbacks := [Identity]); end."
    ));
}

#[test]
fn nested_callbacks_require_evidence_and_matching_signatures() {
    for (statement, message) in [
        (
            "const H := Handler(Callbacks := [Identity]);",
            "Cannot infer",
        ),
        (
            "const H: Handler of integer := Handler(Callbacks := [Text]);",
            "field `Callbacks`",
        ),
    ] {
        let errors = check_errors(&format!(
            "program T; {ROUTINES} type Handler of T = record Callbacks: array of function(Value: T): T; end record;
             begin {statement} end."
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TYPE_MISMATCH && error.message.contains(message)),
            "{errors:#?}"
        );
    }
    let errors = check_errors(&format!(
        "program T; {ROUTINES} type Handler of T = record Callbacks: array of function(Value: T): T; Value: T; end record;
         begin const H := Handler(Callbacks := [Number], Value := 'text'); end."
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
        "{errors:#?}"
    );
}

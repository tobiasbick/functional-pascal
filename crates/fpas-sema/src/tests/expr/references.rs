//! Var modes require caller storage and cannot escape synchronous calls.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_IMMUTABLE_ASSIGNMENT, SEMA_TYPE_MISMATCH};

#[test]
fn explicit_modes_accept_forwarding_and_independent_value_snapshots() {
    check_ok(
        r#"program T;
procedure Inner(var Value: integer; Snapshot: integer); begin Value := Snapshot; end procedure;
procedure Outer(var Value: integer); begin Inner(var Value, Value); end procedure;
begin  var Count: integer := 1; Outer(var Count); end program;"#,
    );
}

#[test]
fn var_modes_reject_missing_or_unexpected_actual_markers() {
    for (formal, actual) in [
        ("var Value: integer", "Count"),
        ("Value: integer", "var Count"),
    ] {
        let errors = check_errors(&format!(
            r#"program T; procedure Use({formal}); begin null; end procedure; begin  var Count: integer := 1; Use({actual}); end program;"#
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
                && error.help.as_ref().is_some_and(|help| help.contains("var"))),
            "{errors:#?}"
        );
    }
}

#[test]
fn var_modes_reject_const_readonly_parameter_and_loop_roots() {
    for source in [
        "program T; procedure Use(var Value: integer); begin null; end procedure; const Fixed: integer := 1; begin Use(var Fixed); end program;",
        "program T; procedure Use(var Value: integer); begin null; end procedure; procedure Caller(Value: integer); begin Use(var Value); end procedure; begin null; end program;",
        "program T; procedure Use(var Value: integer); begin null; end procedure; begin for Index: integer := 0 to 1 do Use(var Index); end for; end program;",
    ] {
        let errors = check_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_IMMUTABLE_ASSIGNMENT),
            "{errors:#?}"
        );
    }
}

#[test]
fn var_modes_reject_duplicate_root_even_for_disjoint_fields_and_indices() {
    for source in [
        r#"program T; type Pair = record A: integer; B: integer; end record; procedure Use(var A: integer; var B: integer); begin null; end procedure; begin  var P: Pair := Pair(A := 1, B := 2); Use(var P.A, var P.B); end program;"#,
        r#"program T; procedure Use(var A: integer; var B: integer); begin null; end procedure; begin  var Items: array of (integer) := [1, 2]; Use(var Items[0], var Items[1]); end program;"#,
    ] {
        let errors = check_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TYPE_MISMATCH && error.message.contains("root")),
            "{errors:#?}"
        );
    }
}

#[test]
fn var_modes_require_exact_storage_types_and_callable_modes() {
    for source in [
        r#"program T; procedure Use(var Value: real); begin null; end procedure; begin  var Count: integer := 1; Use(var Count); end program;"#,
        r#"program T; procedure Use(var Value: integer); begin null; end procedure; begin const F: procedure(Value: integer) := Use; end program;"#,
        r#"program T; procedure Use(Value: integer); begin null; end procedure; begin const F: procedure(var Value: integer) := Use; end program;"#,
    ] {
        let errors = check_errors(source);
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{errors:#?}"
        );
    }
}

#[test]
fn var_parameters_cannot_be_captured_by_named_or_anonymous_routines() {
    for source in [
        "program T; procedure Outer(var Value: integer); procedure Nested(); begin Value := 1; end procedure; begin Nested(); end procedure; begin null; end program;",
        "program T; function Outer(var Value: integer): function(): integer; begin return function(): integer begin return Value; end function; end function; begin null; end program;",
    ] {
        let errors = check_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("capture var parameter")),
            "{errors:#?}"
        );
    }
    check_ok(
        r#"program T; function Outer(var Value: integer): function(): integer; begin const Snapshot: integer := Value; return function(): integer begin return Snapshot; end function; end function; begin null; end program;"#,
    );
}

#[test]
fn var_arguments_cannot_cross_task_boundaries() {
    let errors = check_errors(
        r#"program T; function Use(var Value: integer): integer; begin return Value; end function; begin  var Count: integer := 1; const Pending: task of (integer) := go Use(var Count); end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("task boundary")),
        "{errors:#?}"
    );
}

#[test]
fn var_arguments_reject_string_elements_and_immutable_record_snapshots() {
    for source in [
        r#"program T; procedure Use(var Value: char); begin null; end procedure; begin  var Text: string := 'text'; Use(var Text[0]); end program;"#,
        r#"program T; type Box = record Value: integer; end record; function GetItem(Receiver: Box): integer; begin return Receiver.Value; end function; procedure Use(var Value: integer); begin null; end procedure; begin var Data := Box(Value := 1); const Snapshot := GetItem(Data); Use(var Snapshot); end program;"#,
    ] {
        let errors = check_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_IMMUTABLE_ASSIGNMENT),
            "{errors:#?}"
        );
    }
}

#[test]
fn record_functions_and_callable_fields_require_explicit_var_arguments() {
    for call in ["BoxUse(Data, Count)", "Data.Apply(Count)"] {
        let source = format!(
            r#"program T;
            type Box = record Item: integer; Apply: procedure(var Value: integer); end record;
            procedure BoxUse(Receiver: Box; var Value: integer); begin Value := Value + Receiver.Item; end procedure;
            procedure Increase(var Value: integer); begin Value := Value + 1; end procedure;
            begin var Count := 1; const Data := Box(Item := 2, Apply := Increase); {call}; end program;"#
        );
        let errors = check_errors(&source);
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("explicit `var`")),
            "{errors:#?}"
        );
        check_ok(&source.replace("Count)", "var Count)"));
    }
}

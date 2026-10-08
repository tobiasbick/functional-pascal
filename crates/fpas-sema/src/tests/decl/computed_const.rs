use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION;

#[test]
fn static_labels_ranges_and_dynamic_guards_remain_valid() {
    check_ok(
        "program T; function ReadValue(): integer; begin return 2; end function;
        begin const First: integer := 1; const Last: integer := First + 2;
        case 2 of when First..Last: null; when V if V = ReadValue(): null; end case; end.",
    );
}

#[test]
fn named_nested_routines_preserve_enclosing_const_classification() {
    check_ok("program T; function Outer(): integer;
        function Inner(): integer; begin case 42 of when Fixed: return 42; end case; return 0; end function;
        begin const Fixed: integer := 6 * 7; return Inner(); end function; begin end.");
    let errors = check_errors("program T; function Value(): integer; begin return 42; end function;
        function Outer(): integer;
        function Inner(): integer; begin case 42 of when Computed: return 42; end case; return 0; end function;
        begin const Computed: integer := Value(); return Inner(); end function; begin end.");
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_NON_CONSTANT_EXPRESSION
                && error.message.contains("Computed")),
        "{errors:?}"
    );
}

#[test]
fn runtime_dependencies_are_named_in_value_and_range_diagnostics() {
    for (label, dependency) in [
        ("ReadValue()", "ReadValue"),
        ("Computed", "Computed"),
        ("Derived", "Derived"),
        ("Seed", "Seed"),
        ("0..Computed", "Computed"),
        ("Computed..3", "Computed"),
        ("(ReadValue() + 1)..3", "ReadValue"),
        ("Std.Math.Abs(-1)", "Abs"),
        ("'x'.Length()", "Length"),
        ("'abc'.Length()", "Length"),
    ] {
        let source = format!(
            "program T; uses Std.Math;\n            function ReadValue(): integer; begin return 1; end function;\n            const Seed: integer := ReadValue() - 1; const Computed: integer := ReadValue();\n            const Derived: integer := Computed + 1;\n            begin case 1 of when {label}: null; else null; end case; end."
        );
        let errors = check_errors(&source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_NON_CONSTANT_EXPRESSION
                    && error.message.contains(dependency)),
            "{label}: {errors:#?}"
        );
    }
}

#[test]
fn computed_constant_with_guard_does_not_become_a_pattern_binding() {
    let errors = check_errors(
        "program T; function ReadValue(): integer; begin return 1; end function;
        begin const C: integer := ReadValue(); case 1 of when C if C > 0: null; end case; end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_NON_CONSTANT_EXPRESSION && error.message.contains("C")),
        "{errors:?}"
    );
}

#[test]
fn local_const_has_immutable_binding_and_lexical_scope() {
    check_errors("program T; begin const C: integer := 1; C := 2; end.");
    check_errors(
        "program T; begin if true then const C: integer := 1; end if; const X: integer := C; end.",
    );
    check_ok(
        "program T; begin const C: integer := 1; begin const C: integer := 2; end; const X: integer := C; end.",
    );
}

#[test]
fn constant_aggregates_preserve_static_forms_and_hidden_default_dependencies() {
    check_ok(
        "program T;
        type Point = record X: integer := 1; end record;
        const Values: array of integer := [1, 2];
        const Mapping: dict of string to integer := ['a': 1];
        const Wrapped: Option of integer := Some(2);
        const Answer: result of integer, string := Ok(3);
        const P: Point := record end;
        const Updated: Point := P with X := 2; end with;
        begin case 1 of when P.X: null; end case; end.",
    );
    let errors = check_errors(
        "program T;
        function Value(): integer; begin return 1; end function;
        type Point = record X: integer := Value(); end record;
        const P: Point := record end;
        begin case 1 of when P.X: null; end case; end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_NON_CONSTANT_EXPRESSION && error.message.contains("P")),
        "{errors:?}"
    );
}

#[test]
fn const_initializers_keep_bare_task_and_callable_capture_metadata() {
    check_ok(
        "program T; uses Std.Tasks; function Work(): integer; begin return 7; end function;
        begin const Pending: task := go Work(); const Value: integer := Wait(Pending); end.",
    );
    check_ok(
        "program T; begin const ReadValue: function(): integer := function(): integer begin return 7; end function; discard ReadValue; end.",
    );
    check_errors("program T; begin var X: integer := 0;
        const Change: function(): integer := function(): integer begin X := X + 1; return X; end function;
        go Change(); end.");
}

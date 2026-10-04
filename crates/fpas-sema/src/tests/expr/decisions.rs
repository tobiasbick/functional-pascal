//! Value decision typing, contextual constructors, coverage, and callable state.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_NON_EXHAUSTIVE_CASE, SEMA_TYPE_MISMATCH};

#[test]
fn decisions_propagate_expected_types_through_generic_constructors_and_wrappers() {
    check_ok(
        "program Main;\n        type Choice of (T) = enum Present(Value: T); Missing; end enum;\n        type Box of (T) = record Value: T; end record;\n        function Select(Flag: boolean): Choice of (integer); begin return if Flag then Choice.Missing else Choice.Present(42) end if; end function;\n        var Values: array of (Choice of (integer)) := [if true then Choice.Missing else Choice.Present(42) end if];\n        var Wrapped: Option of (Choice of (integer)) := Option.Some(if true then Choice.Missing else Choice.Present(42) end if);\n        var Number: Box of (integer) := Box(Value := if true then 42 else 0 end if);\n        begin var Answer: integer := case Select(false) of\n          when Choice.Present(const Value): if Value > 0 then Value else 0 end if;\n          when Choice.Missing: 0;\n        end case; end program;",
    );
}

#[test]
fn branch_and_call_argument_order_do_not_choose_generic_constructor_types() {
    for (first, second) in [
        ("Choice.Missing", "Choice.Present(42)"),
        ("Choice.Present(42)", "Choice.Missing"),
    ] {
        check_ok(&format!("program Main;
            type Choice of (T) = enum Present(Value: T); Missing; end enum;
            function Identity of (U)(Value: U): U; begin return Value; end function;
            procedure Accept of (U)(Left: Choice of (U); Right: Choice of (U)); begin null; end procedure;
            begin var Item: Choice of (integer) := Identity(if true then {first} else {second} end if);
              Accept({first}, {second}); end program;"));
    }
}

#[test]
fn branches_reject_incompatible_values_procedure_calls_and_unresolved_collections() {
    for expression in [
        "if true then 1 else 'one' end if",
        "if true then 1 else 2.0 end if",
        "if true then Nothing() else Nothing() end if",
        "if true then [] else [] end if",
    ] {
        let errors = check_errors(&format!(
            "program Main; procedure Nothing(); begin null; end procedure;
            function Identity of (T)(Value: T): T; begin return Value; end function;
            begin discard Identity({expression}); end program;"
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{expression}: {errors:#?}"
        );
    }
}

#[test]
fn scalar_case_values_require_fallback_unless_boolean_coverage_is_complete() {
    check_ok("program Main; begin var Value: integer := case true of when true: 42; when false: 0; end case;
        var Other: integer := case 42 of when 1..10: 0; else 42; end case; end program;");
    let errors = check_errors(
        "program Main; begin var Value: integer := case 42 of when 42: 42; end case; end program;",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_NON_EXHAUSTIVE_CASE),
        "{errors:#?}"
    );
}

#[test]
fn value_cases_reject_closed_fallbacks_and_mismatched_or_unresolved_branches() {
    for selection in [
        "case Item of when Choice.Present(const Value): Value; else 0; end case",
        "case Item of when Choice.Present(const Value): Value; when Choice.Missing: 'missing'; end case",
        "case Item of when Choice.Present(const Value): []; when Choice.Missing: []; end case",
        "case true of when true: 1; when false: 0; else 2; end case",
    ] {
        let errors = check_errors(&format!(
            "program Main;
            type Choice = enum Present(Value: integer); Missing; end enum;
            var Item: Choice := Choice.Present(42);
            function Identity of (T)(Value: T): T; begin return Value; end function;
            begin discard Identity({selection}); end program;"
        ));
        assert!(!errors.is_empty(), "accepted {selection}");
        assert!(
            errors
                .iter()
                .all(|error| error.code
                    != fpas_diagnostics::codes::INTERNAL_COMPILER_INVARIANT_FAILURE),
            "{errors:#?}"
        );
    }
}

#[test]
fn case_value_arms_share_nested_binding_rules_and_reject_guard_only_coverage() {
    let errors = check_errors(
        "program Main;
        type Choice = enum Present(Value: integer); Missing; end enum;
        var Item: Choice := Choice.Present(42);
        begin var Value: integer := case Item of
          when Choice.Present(const Value) if Value > 0: Value;
          when Choice.Missing: 0;
        end case; end program;",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_NON_EXHAUSTIVE_CASE),
        "{errors:#?}"
    );
}

#[test]
fn selected_and_pattern_bound_stateful_callables_cannot_cross_task_boundaries() {
    for selected in [
        "if true then Action else Action end if",
        "case Item of when Choice.Present(const Callback): Callback; end case",
    ] {
        let errors = check_errors(&format!("program Main;
            type Choice of (T) = enum Present(Value: T); end enum;
            begin mutable var State: integer := 0;
              var Action: function(): integer := function(): integer begin State := State + 1; return State; end function;
              var Item: Choice of (function(): integer) := Choice.Present(Action);
              var Selected: function(): integer := {selected}; var Handle: task := go Selected();
            end program;"));
        assert!(
            errors
                .iter()
                .any(|error| error.code == fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE),
            "{selected}: {errors:#?}"
        );
    }
}

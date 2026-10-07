//! Writable bindings and preservation of former locally writable parameters.

use super::*;

#[test]
fn writable_bindings_and_parameter_copies_run_in_the_fpas_suite() {
    assert_succeeds(include_str!(
        "../../../../tests/runner/binding_keyword_switch_test.fpas"
    ));
}

#[test]
fn const_parameters_and_loop_variables_reject_writes_with_hints() {
    for (source, hint) in [
        (
            "program T; begin const X: integer := 1; X := 2; end.",
            "var",
        ),
        (
            "program T; procedure P(X: integer); begin X := 2; end procedure; begin end.",
            "local copy",
        ),
        (
            "program T; begin for I: integer := 1 to 2 do I := 3; end for; end.",
            "Loop variables",
        ),
        (
            "program T; begin for I: integer in [1, 2] do I := 3; end for; end.",
            "Loop variables",
        ),
    ] {
        let errors = crate::compile(&parse_ok(source)).expect_err("binding is read-only");
        assert!(
            errors.iter().any(|error| error
                .help
                .as_deref()
                .is_some_and(|text| text.contains(hint))),
            "{errors:#?}"
        );
    }
}

#[test]
fn read_only_capture_of_a_parameter_copy_stays_task_bound() {
    let program = parse_ok(
        "program T;
        procedure CheckCopy(Value: integer);
        begin var LocalValue: integer := Value;
          const Reader: function(): integer := function(): integer begin return LocalValue; end function;
          const Work: task := go Reader();
        end procedure;
        begin CheckCopy(1); end.",
    );
    let errors = crate::compile(&program).expect_err("shared cell cannot cross task boundary");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("task-bound")),
        "{errors:#?}"
    );
}

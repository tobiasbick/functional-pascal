use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{
    SEMA_TASK_BOUND_CALLABLE, SEMA_TYPE_MISMATCH, SEMA_WRONG_ARGUMENT_COUNT,
};

const DECLARATIONS: &str = r#"
type Handler = function(Input: integer): integer;
function Make(): Handler;
begin return function(Other: integer): integer begin return Other; end function; end function;
procedure Action(); begin null; end procedure;
function MakeAction(): procedure(); begin return Action; end function;
"#;

#[test]
fn procedure_values_can_be_stored_and_called_without_a_value() {
    check_ok(&format!(
        "program P; {DECLARATIONS} begin var Actions: array of procedure() := [MakeAction()]; Actions[0](); (Action)(); MakeAction()(); end program;"
    ));
}

#[test]
fn arbitrary_targets_check_arity_types_and_callability() {
    for (expression, code, message) in [
        ("Make()()", SEMA_WRONG_ARGUMENT_COUNT, "expects 1 arguments"),
        (
            "Make()(1, 2)",
            SEMA_WRONG_ARGUMENT_COUNT,
            "expects 1 arguments",
        ),
        ("Make()('text')", SEMA_TYPE_MISMATCH, "argument 1"),
        ("(42)()", SEMA_TYPE_MISMATCH, "not callable"),
    ] {
        let errors = check_errors(&format!(
            "program P; {DECLARATIONS} begin discard {expression}; end program;"
        ));
        assert_eq!(errors.len(), 1, "{expression}: {errors:#?}");
        assert_eq!(errors[0].code, code);
        assert!(errors[0].message.contains(message), "{errors:#?}");
    }
}

#[test]
fn procedure_calls_cannot_initialize_bindings_or_be_discarded() {
    for statement in [
        "var Value: integer := MakeAction()();",
        "discard MakeAction()();",
        "discard Action();",
        "var Value: integer := MakeAction()().Field;",
    ] {
        let errors = check_errors(&format!(
            "program P; {DECLARATIONS} begin {statement} end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("does not return a value")),
            "{errors:#?}"
        );
    }
}

#[test]
fn all_function_statements_need_a_consumer() {
    for statement in [
        "Make();",
        "Make()(1);",
        "(Make())(1);",
        "[Make()][0](1);",
        "Math.Abs(-1);",
    ] {
        let errors = check_errors(&format!(
            "program P; uses Std.Math as Math; {DECLARATIONS} begin {statement} end program;"
        ));
        assert_eq!(errors.len(), 1, "{statement}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH);
        assert_eq!(errors[0].message, "Function result must be consumed");
        assert!(
            errors[0]
                .help
                .as_deref()
                .is_some_and(|help| help.contains("discard"))
        );
    }
}

#[test]
fn discard_accepts_ordinary_values_and_callable_values() {
    check_ok(&format!(
        "program P; {DECLARATIONS} begin discard 42; discard [1, 2]; discard Some(1); discard Make(); discard Make()(1); discard Action; end program;"
    ));
}

#[test]
fn discard_rejects_nested_task_handles() {
    for value in [
        "Handle",
        "Queue",
        "[Handle]",
        "Some(Handle)",
        "Ok(Handle)",
        "['task': Handle]",
        "record Child := Handle; end record",
        "Payload.Child(Handle)",
    ] {
        let errors = check_errors(&format!(
            r#"program P; uses Std.Tasks as Tasks;
type Payload = enum Child(Handle: task of integer); end enum;
function Work(): integer; begin return 1; end function;
begin var Handle: task := go Work(); var Queue: channel of task of integer := Tasks.CreateChannel(1); discard {value}; end program;"#
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("Cannot discard a task handle")),
            "{value}: {errors:#?}"
        );
    }
}

#[test]
fn postfix_calls_preserve_task_bound_target_checks() {
    for call in [
        "(Action)()",
        "Actions[0]()",
        "(Actions)[0]()",
        "([Action])[0]()",
    ] {
        let errors = check_errors(&format!(
            r#"program P; begin
mutable var Count: integer := 0;
var Action: procedure() := procedure() begin Count := Count + 1; end procedure;
var Actions: array of procedure() := [Action];
go {call}; end program;"#
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TASK_BOUND_CALLABLE),
            "{call}: {errors:#?}"
        );
    }
}

#[test]
fn invalid_target_still_checks_arguments_without_suffix_cascades() {
    let errors = check_errors("program P; begin discard (1)(Missing).Other(); end program;");
    assert_eq!(errors.len(), 2, "{errors:#?}");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("not callable"))
    );
    assert!(errors.iter().any(|error| error.message.contains("Missing")));
}

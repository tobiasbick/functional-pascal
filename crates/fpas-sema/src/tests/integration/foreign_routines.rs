//! Pascal function-name assignment and Delphi `Exit`: one diagnostic each, recovery, and no
//! false positives.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

use super::*;
use fpas_diagnostics::codes::{SEMA_IMMUTABLE_ASSIGNMENT, SEMA_UNKNOWN_NAME};

/// A later, unrelated error that analysis must still report.
const LATER_ERROR: &str = "begin\n  Foo();\nend.";

#[test]
fn assignment_to_function_name_is_reported_once() {
    let source = format!(
        "program P;\n\nfunction F(): integer;\nbegin\n  F := 1;\n  return 2;\nend function;\n\n{LATER_ERROR}"
    );
    let errors = check_errors(&source);
    assert_eq!(errors.len(), 2, "{errors:#?}");
    let error = &errors[0];
    assert_eq!(error.code, SEMA_IMMUTABLE_ASSIGNMENT);
    assert!(
        error
            .message
            .contains("assignment to function name `F` does not set its result"),
        "{error:#?}"
    );
    assert_eq!(error.found.as_deref(), Some("F := …"));
    assert_eq!(error.expected.as_deref(), Some("return …"));
    assert!(
        error
            .help
            .as_deref()
            .is_some_and(|help| help.contains("FPAS writes `return …`")),
        "{error:#?}"
    );
    assert_eq!(errors[1].code, SEMA_UNKNOWN_NAME);
}

#[test]
fn function_name_assignment_still_checks_the_value_type() {
    let errors = check_errors(
        "program P;\n\nfunction F(): integer;\nbegin\n  F := 'text';\n  return 2;\nend function;\n\nbegin\nend.",
    );
    assert_eq!(errors.len(), 2, "{errors:#?}");
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_IMMUTABLE_ASSIGNMENT)
    );
}

#[test]
fn local_shadowing_the_function_name_is_assignable() {
    check_ok(
        "program P;\n\nfunction F(): integer;\nbegin\n  var F: integer := 0;\n  F := 1;\n  return F;\nend function;\n\nbegin\nend.",
    );
}

#[test]
fn delphi_exit_names_return() {
    let source = format!(
        "program P;\n\nfunction F(): integer;\nbegin\n  Exit(1);\nend function;\n\n{LATER_ERROR}"
    );
    let errors = check_errors(&source);
    assert_eq!(errors.len(), 2, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_UNKNOWN_NAME);
    assert!(
        errors[0]
            .help
            .as_deref()
            .is_some_and(|help| help.contains("Delphi `Exit`")
                && help.contains("FPAS writes `return Value;`")),
        "{errors:#?}"
    );
}

#[test]
fn user_routine_named_exit_is_not_reported() {
    check_ok(
        "program P;\n\nprocedure Exit(Code: integer);\nbegin\n  return;\nend procedure;\n\nbegin\n  Exit(1);\nend.",
    );
}

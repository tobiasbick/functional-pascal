//! Static parameter labels for fixed-arity polymorphic standard operations.
//!
//! Types are validated by their operation-specific checkers; these labels also
//! appear in the intrinsic API consumed by editor tools.
//! **Documentation:** `docs/pascal/std/README.md`

use fpas_std::std_symbols as s;

/// Returns the published parameter labels used by a registered polymorphic operation.
pub(in crate::std_registry) fn parameter_names(name: &str) -> &'static [&'static str] {
    match name {
        s::STD_MATH_ABS | s::STD_MATH_SIGN => &["N"],
        s::STD_MATH_MIN | s::STD_MATH_MAX => &["A", "B"],
        s::STD_MATH_CLAMP => &["V", "Lo", "Hi"],
        s::STD_TEST_ASSERT_EQUALS => &["Expected", "Actual"],
        s::STD_TASK_START_TASK_IN_GROUP => &["Group", "Work"],
        s::STD_TASK_START_SUPERVISED_TASK => &["Group", "Work", "RetryLimit", "BackoffMillis"],
        s::STD_TASK_CREATE_CHANNEL => &["Capacity"],
        s::STD_TASK_SEND | s::STD_TASK_TRY_SEND => &["Queue", "Value"],
        s::STD_TASK_SEND_WITH_CANCELLATION => &["Queue", "Value", "Token"],
        s::STD_TASK_SEND_WITH_TIMEOUT => &["Queue", "Value", "TimeoutMillis"],
        s::STD_TASK_RECEIVE | s::STD_TASK_TRY_RECEIVE | s::STD_TASK_CLOSE_CHANNEL => &["Queue"],
        s::STD_TASK_RECEIVE_WITH_CANCELLATION => &["Queue", "Token"],
        s::STD_TASK_RECEIVE_WITH_TIMEOUT => &["Queue", "TimeoutMillis"],
        s::STD_TASK_WAIT => &["Handle"],
        s::STD_TASK_WAIT_ALL | s::STD_TASK_WAIT_ANY => &["Tasks"],
        s::STD_TASK_WAIT_ANY_WITH_TIMEOUT => &["Tasks", "TimeoutMillis"],
        s::STD_TASK_WAIT_ANY_WITH_CANCELLATION => &["Tasks", "Token"],
        s::STD_TASK_RECEIVE_CASE => &["Queue", "Callback"],
        s::STD_TASK_SEND_CASE => &["Queue", "Value", "Callback"],
        s::STD_TASK_TASK_CASE => &["Handle", "Callback"],
        s::STD_TASK_TIMER_CASE => &["Milliseconds", "Callback"],
        s::STD_TASK_CANCELLATION_CASE => &["Token", "Callback"],
        _ => unreachable!("unregistered polymorphic operation: {name}"),
    }
}

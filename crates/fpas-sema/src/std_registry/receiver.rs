//! Writable receiver mode of built-in array operations.
//!
//! The ordinary call checker remains responsible for every remaining argument.

use crate::types::ParamMode;

/// Parameter mode of an intrinsic's first argument, including an implicit dot receiver.
///
/// Caller-mutating array operations use `var` storage checks. Operations on
/// channels, tasks, and host handles change shared runtime state, not the caller's binding.
/// **Documentation:** `docs/pascal/language/types/array/mutating.md`
#[must_use]
pub fn intrinsic_std_receiver_mode(name: &str) -> ParamMode {
    super::native_operation_by_implementation(name)
        .map_or(ParamMode::Value, |operation| operation.receiver_mode())
}

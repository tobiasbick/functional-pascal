//! Writable receiver mode of built-in array operations.
//!
//! The ordinary call checker remains responsible for every remaining argument.

use crate::types::ParamMode;
use fpas_std::std_symbols as s;

/// Parameter mode of an intrinsic's first argument, including an implicit dot receiver.
///
/// Caller-mutating array operations use `var` storage checks. Operations on
/// channels, tasks, and host handles change shared runtime state, not the caller's binding.
/// **Documentation:** `docs/pascal/language/types/array/mutating.md`
#[must_use]
pub fn intrinsic_std_receiver_mode(name: &str) -> ParamMode {
    if [s::STD_ARRAY_PUSH, s::STD_ARRAY_POP]
        .iter()
        .any(|known| name.eq_ignore_ascii_case(known))
    {
        ParamMode::Var
    } else {
        ParamMode::Value
    }
}

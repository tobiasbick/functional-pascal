//! `var` parameters and arguments: argument validity, aliasing, and lifetime rules.
//!
//! **Documentation:** `docs/pascal/language/functions/var-parameters.md`

mod arguments;
mod escapes;
mod pending;
mod storage;

pub(crate) use pending::PendingRoutineUse;

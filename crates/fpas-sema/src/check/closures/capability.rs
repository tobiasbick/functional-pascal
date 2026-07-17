//! Closure capability checks (task-bound transfer).
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

use super::CaptureBinding;

/// Returns whether a closure is task-bound (may not cross task boundaries freely).
///
/// Mutable (cell) captures make a closure task-bound.
///
/// **Documentation:** `docs/pascal/language/functions/closures.md`
#[must_use]
pub fn task_bound_from_captures(captures: &[CaptureBinding]) -> bool {
    captures.iter().any(|c| c.mutable)
}

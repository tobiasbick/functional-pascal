//! Closure semantic analysis: captures and capabilities.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

mod capability;
mod capture;

use std::collections::HashMap;

pub use capability::task_bound_from_captures;
pub use capture::{CaptureBinding, collect_captures};

/// Semantic metadata for one closure expression (keyed by [`crate::expr_lookup_key`]).
///
/// **Documentation:** `docs/pascal/language/functions/closures.md`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureInfo {
    /// Free variables captured from enclosing scopes.
    pub captures: Vec<CaptureBinding>,
    /// `true` when any capture is mutable or already task-bound; restricts task spawning.
    pub task_bound: bool,
    /// Compiler-facing synthetic routine name for this closure body.
    pub synthetic_name: String,
}

/// Maps closure expression identity to [`ClosureInfo`].
pub type ClosureInfoMap = HashMap<usize, ClosureInfo>;

/// Capture metadata for a named nested routine (keyed by canonical routine name).
///
/// **Documentation:** `docs/pascal/language/functions/closures.md`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestedRoutineCaptureInfo {
    /// Free variables captured from enclosing scopes.
    pub captures: Vec<CaptureBinding>,
    /// `true` when any capture is mutable or already task-bound.
    pub task_bound: bool,
}

/// Maps nested routine names to their capture metadata.
pub type NestedRoutineCaptureMap = HashMap<String, NestedRoutineCaptureInfo>;

/// Build [`ClosureInfo`] from analyzed captures.
#[must_use]
pub fn closure_info_from_captures(
    synthetic_name: String,
    captures: Vec<CaptureBinding>,
) -> ClosureInfo {
    let task_bound = task_bound_from_captures(&captures);
    ClosureInfo {
        captures,
        task_bound,
        synthetic_name,
    }
}

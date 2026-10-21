//! Closure semantic analysis: captures and capabilities.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

mod capability;
mod capture;
mod captured_writes;
mod dependencies;
mod direct_capabilities;
mod pending_assignments;
mod pending_capabilities;
mod pending_values;

use std::collections::HashMap;

pub use capability::capability_from_captures;
pub use capture::{CaptureAnalysis, CaptureBinding, collect_captures};
pub(crate) use dependencies::{CaptureDependencies, CaptureOwner};

/// Semantic metadata for one closure expression (keyed by [`crate::expr_lookup_key`]).
///
/// **Documentation:** `docs/pascal/language/functions/closures.md`
#[derive(Debug, Clone, PartialEq)]
pub struct ClosureInfo {
    /// Free variables captured from enclosing scopes.
    pub captures: Vec<CaptureBinding>,
    /// Transfer capability derived from the captured values.
    pub capability: crate::TransferCapability,
}

/// Maps closure expression identity to [`ClosureInfo`].
pub type ClosureInfoMap = HashMap<usize, ClosureInfo>;

/// Capture metadata for a named nested routine (keyed by AST declaration identity).
///
/// **Documentation:** `docs/pascal/language/functions/closures.md`
#[derive(Debug, Clone, PartialEq)]
pub struct NestedRoutineCaptureInfo {
    /// Free variables captured from enclosing scopes.
    pub captures: Vec<CaptureBinding>,
    /// Transfer capability derived from the captured values.
    pub capability: crate::TransferCapability,
}

/// Maps stable in-pipeline routine declaration identities to capture metadata.
pub type NestedRoutineCaptureMap = HashMap<usize, NestedRoutineCaptureInfo>;

/// Build [`ClosureInfo`] from analyzed captures.
#[must_use]
pub fn closure_info_from_captures(captures: Vec<CaptureBinding>) -> ClosureInfo {
    let capability = capability_from_captures(&captures);
    ClosureInfo {
        captures,
        capability,
    }
}

//! Immutable stopped-state snapshots and bounded variable expansion.

mod model;
mod render;
mod snapshot;

pub use model::{
    DebugFrame, DebugInspectionLimits, DebugScope, DebugScopeKind, DebugVariable, Paginated,
};
pub(super) use snapshot::InspectionSnapshot;

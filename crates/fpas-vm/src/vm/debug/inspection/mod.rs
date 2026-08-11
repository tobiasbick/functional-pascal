//! Immutable stopped-state snapshots and bounded variable expansion.

mod capture;
mod handles;
mod model;
mod render;
mod snapshot;
mod targets;

pub use model::{
    DebugFrame, DebugInspectionLimits, DebugScope, DebugScopeKind, DebugVariable, Paginated,
};
pub(super) use snapshot::InspectionSnapshot;
pub(super) use targets::{MutationPath, MutationRoot, MutationTarget};

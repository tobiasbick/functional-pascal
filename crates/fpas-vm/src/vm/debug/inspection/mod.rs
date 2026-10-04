//! Immutable stopped-state snapshots and bounded variable expansion.

mod capture;
mod capture_sources;
mod handles;
mod model;
mod mutation_targets;
mod render;
mod snapshot;
mod storage;
mod targets;
mod typed_bindings;

pub use model::{
    DebugFrame, DebugInspectionLimits, DebugScope, DebugScopeKind, DebugVariable, Paginated,
};
pub(super) use snapshot::InspectionSnapshot;
pub(super) use storage::logical_global_type;
pub(super) use targets::{
    MutationPath, MutationRoot, MutationTarget, PayloadError, active_label, resolve_payload,
};

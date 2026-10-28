//! Portable source-level debugger metadata and call-safety analysis.

mod callable;
mod effects;
mod projection;
mod types;

pub use callable::{
    DebugBinding, DebugBindingKind, DebugCaptureKind, DebugCaptureSource, DebugScope,
    DebugSourceLocation, FunctionDebugInfo, SequencePoint,
};
pub use effects::{
    DebugEffectSet, FunctionEffectSummary, analyze_debug_effects, intrinsic_debug_effects,
};
pub use projection::{
    DebugProjection, DebugProjectionKind, MAX_DEBUG_PROJECTION_DEPTH,
    MAX_DEBUG_PROJECTION_NAME_BYTES, MAX_DEBUG_PROJECTION_NODES,
};
pub use types::DebugType;

//! Portable source-level debugger metadata and call-safety analysis.

mod callable;
mod effects;

pub use callable::{
    DebugBinding, DebugBindingKind, DebugScope, DebugSourceLocation, FunctionDebugInfo,
    SequencePoint,
};
pub use effects::{
    DebugEffectSet, FunctionEffectSummary, analyze_debug_effects, intrinsic_debug_effects,
};

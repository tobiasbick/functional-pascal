//! Opt-in scalar global/local observation with explicit declaration lifetimes.

mod binding;
mod capture_binding;
mod capture_visibility;
mod history;
mod local_admission;
mod local_binding;
mod local_history;
mod local_state;
mod local_value;
mod local_worker;
mod model;
mod pending;
mod selector;
mod stored_path;
mod worker;
pub(in crate::execution::vm) use local_state::LocalTraceState;

pub(in crate::execution::vm) use history::TraceHistory;
pub use model::{
    DEFAULT_TRACE_EVENTS, DebugTraceCaptureSource, DebugTraceEvent, DebugTraceIdentity,
    DebugTraceLocation, DebugTracePathStep, DebugTraceReport, DebugTraceWatch, MAX_TRACE_BYTES,
    MAX_TRACE_EVENTS,
};

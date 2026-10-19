//! Opt-in scalar global/local observation with explicit declaration lifetimes.

mod binding;
mod history;
mod local_binding;
mod local_history;
mod local_state;
mod local_worker;
mod model;
mod pending;
mod worker;
pub(in crate::vm) use local_state::LocalTraceState;

pub(in crate::vm) use history::TraceHistory;
pub use model::{
    DEFAULT_TRACE_EVENTS, DebugTraceEvent, DebugTraceIdentity, DebugTraceLocation,
    DebugTraceReport, DebugTraceWatch, MAX_TRACE_BYTES, MAX_TRACE_EVENTS,
};

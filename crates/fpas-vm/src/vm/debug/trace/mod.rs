//! Opt-in, per-commit global scalar observation without physical debugger stops.

mod binding;
mod history;
mod model;
mod worker;

pub(in crate::vm) use history::TraceHistory;
pub use model::{
    DEFAULT_TRACE_EVENTS, DebugTraceEvent, DebugTraceIdentity, DebugTraceLocation,
    DebugTraceReport, DebugTraceWatch, MAX_TRACE_BYTES, MAX_TRACE_EVENTS,
};

//! Detached trace records and their canonical escaped-byte accounting.

use crate::{FailureValue, snapshot_value};
use serde::{Serialize, Serializer};
use std::collections::BTreeSet;

/// Maximum escaped trace control data and complete events.
pub const MAX_TRACE_BYTES: usize = 2 * 1024 * 1024;
/// Maximum configurable complete event count.
pub const MAX_TRACE_EVENTS: usize = 10_000;
/// Default complete event count.
pub const DEFAULT_TRACE_EVENTS: usize = 1000;
pub(super) const MAX_SCALAR_BYTES: usize = 65_536;
pub(super) const MAX_WORK_BYTES: usize = 16 * 1024 * 1024;

/// A pre-execution metadata binding, independent of stop-local inspection handles.
#[derive(Debug, Clone, Serialize)]
pub struct DebugTraceWatch {
    /// Zero-based watch identity in request order.
    pub id: usize,
    /// First authored selector spelling for this binding.
    pub request: String,
    /// Canonical metadata global and stored-field path.
    pub name: String,
    /// Executable-global root and fixed stored-field slots.
    pub identity: DebugTraceIdentity,
    #[serde(rename = "type")]
    /// Declared scalar source type name.
    pub type_name: String,
    #[serde(skip)]
    pub(super) ty: fpas_bytecode::DebugTypeId,
}

/// Fixed executable-global root and positional stored record-field path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DebugTraceIdentity {
    /// Zero-based executable global slot.
    pub global_slot: usize,
    /// Stored record field indexes in root-to-leaf order.
    pub field_slots: Vec<usize>,
}

/// The actual writing instruction, including callbacks with a separate worker.
#[derive(Debug, Serialize)]
pub struct DebugTraceLocation {
    /// Portable path from the verified source map.
    pub source: String,
    /// One-based source line of the actual store instruction.
    pub line: u32,
    /// One-based source column of the actual store instruction.
    pub column: u32,
}

/// One complete committed initialization or scalar change.
#[derive(Debug, Serialize)]
pub struct DebugTraceEvent {
    /// One-based monotonic event order across the invocation.
    pub sequence: usize,
    /// Zero-based identity in the watch list.
    pub watch_id: usize,
    /// Initialization or consecutive stored-value change.
    pub kind: &'static str,
    /// Actual writing task identity.
    pub task_id: u64,
    /// Logical writing routine name when retained in metadata.
    pub function: Option<String>,
    /// Actual store instruction location, never the following stop.
    pub location: Option<DebugTraceLocation>,
    #[serde(serialize_with = "value")]
    /// Detached scalar value before the successful commit.
    pub old_value: FailureValue,
    #[serde(serialize_with = "value")]
    /// Detached scalar value after the successful commit.
    pub new_value: FailureValue,
}

/// Bounded history through the actual final execution boundary, not future code.
#[derive(Debug, Default, Serialize)]
pub struct DebugTraceReport {
    /// All authored trace requests in invocation order.
    pub requested: Vec<String>,
    /// Deduplicated pre-execution metadata bindings.
    pub watches: Vec<DebugTraceWatch>,
    /// Earliest complete ordered event prefix.
    pub events: Vec<DebugTraceEvent>,
    /// Every qualifying change through the execution end was retained.
    pub complete: bool,
    /// Explicit history loss or scalar presentation truncation reasons.
    pub reasons: BTreeSet<&'static str>,
    /// Number of complete retained events.
    pub event_count: usize,
    /// Exact qualifying count through the actual end, or unavailable.
    pub total_count: Option<usize>,
}

fn value<S: Serializer>(value: &FailureValue, serializer: S) -> Result<S::Ok, S::Error> {
    snapshot_value::Value::new(value).serialize(serializer)
}

/// Counts the same serializer used for publication without allocating JSON bytes.
pub(super) fn encoded_size(value: &impl Serialize) -> usize {
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self.0.saturating_add(bytes.len());
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter(0);
    if serde_json::to_writer(&mut counter, value).is_err() {
        return usize::MAX;
    }
    counter.0
}

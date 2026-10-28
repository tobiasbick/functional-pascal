//! Shallow runtime views and bounded presentation records.

use super::ValueOrigin;
use crate::{MemoryDomain, Value};
use std::collections::HashSet;

/// Shallow retained runtime storage for lazy inspection, not a detached snapshot.
#[derive(Clone)]
pub(in crate::execution::vm) struct RetainedValue<O: ValueOrigin> {
    /// Binding or child display name.
    pub name: String,
    /// Explicit authority for checked cell reads; absent for detached scalar observations.
    pub domain: Option<MemoryDomain>,
    /// Initialized runtime storage, retained shallowly for lazy expansion.
    pub value: Option<Value>,
    /// Declared or runtime display type.
    pub type_name: String,
    /// Storage or presentation marker supplied by the caller.
    pub presentation_hint: Option<String>,
    /// Child-edge depth from the binding root.
    pub depth: usize,
    /// Cell identities already visited on this path, never included in output.
    pub visited_cells: HashSet<usize>,
    /// Portable type used to identify typed child paths.
    pub debug_type: Option<crate::DebugTypeId>,
    /// Caller-owned live storage adapter; absent from detached snapshots.
    pub origin: O,
}

/// One bounded summary with lazy retained children.
pub(in crate::execution::vm) struct RenderedValue<O: ValueOrigin> {
    /// Bounded non-recursive display.
    pub summary: String,
    /// Source or runtime type display.
    pub type_name: String,
    /// Bounded lazy children in runtime order.
    pub children: Vec<RetainedValue<O>>,
    /// Complete named child count.
    pub named_children: usize,
    /// Complete indexed child count.
    pub indexed_children: usize,
    /// Optional storage, opacity, or truncation marker.
    pub presentation_hint: Option<String>,
}

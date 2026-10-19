//! Bounded source-checked projections retained independently of erased layouts.

use crate::{DebugTypeId, StringId};

/// Maximum owned projection depth, including unsupported terminal nodes.
pub const MAX_DEBUG_PROJECTION_DEPTH: usize = 32;
/// Maximum number of nodes in one source binding's projection.
pub const MAX_DEBUG_PROJECTION_NODES: usize = 4096;
/// Maximum cumulative nominal-name UTF-8 bytes in one projection.
pub const MAX_DEBUG_PROJECTION_NAME_BYTES: usize = 65_536;

/// Concrete source type of a stored path, separate from its operational layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugProjection {
    /// Concrete portable type for this source path.
    pub ty: DebugTypeId,
    /// Nominal source name, independently retained for each nested path.
    pub type_name: Option<StringId>,
    /// Bounded source-checked structure.
    pub kind: DebugProjectionKind,
}

/// Stored structures admitted by scalar tracing; other values stay unsupported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugProjectionKind {
    /// Unknown or unsupported source type; never eligible for scalar tracing.
    Unsupported,
    /// Bounded tail using only the existing operational type graph, without refinement.
    Operational,
    /// Supported scalar leaf.
    Scalar,
    /// Source-checked array element.
    Array(Box<DebugProjection>),
    /// Fields in the operational record layout's slot order.
    Record(Vec<DebugProjection>),
    /// Writable parameter's source-checked referent.
    Reference(Box<DebugProjection>),
}

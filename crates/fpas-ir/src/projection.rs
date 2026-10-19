//! Source-checked debug projections independent of operational type erasure.

use crate::TypeId;

/// One node of a bounded, acyclic source-type projection tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugProjection {
    /// Concrete type used to validate this node's runtime value or layout.
    pub ty: TypeId,
    /// Declared nominal scalar name, independent from formatted container names.
    pub type_name: Option<String>,
    /// Supported stored structure or explicit unsupported metadata.
    pub kind: DebugProjectionKind,
}

/// Stored shapes available to exact scalar trace selectors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugProjectionKind {
    /// Bounded semantic expansion tail; only existing operational types remain available.
    Operational,
    /// Unresolved, unsupported or bounded expansion tail; never a traceable leaf.
    Unsupported,
    /// Concrete supported scalar.
    Scalar,
    /// Element projection shared by all positions in an array.
    Array(Box<DebugProjection>),
    /// Child projections in exact operational field-slot order.
    Record(Vec<DebugProjection>),
    /// Referent of a source `var` parameter.
    Reference(Box<DebugProjection>),
}

//! Sidecar reuse and invalidation classification.

use crate::{CompiledUnit, DependencyIdentity, ExpectedUnitIdentity, FormatError};

/// Why an otherwise readable sidecar must be rebuilt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidationReason {
    /// Canonical unit name differs.
    UnitName,
    /// Source content changed.
    Source,
    /// Semantic or code-generation options changed.
    Options,
    /// Direct dependency set or one dependency interface changed.
    Dependencies,
}

/// Why a readable sidecar cannot be consumed by this compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncompatibilityReason {
    /// Envelope format version is not supported.
    FormatVersion(u16),
    /// Compiler build identity changed.
    Compiler,
    /// Executable bytecode format changed.
    Bytecode,
}

/// Result of loading and validating a source-adjacent sidecar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidecarLoad {
    /// Sidecar matches every current input and may be reused.
    Reusable(Box<CompiledUnit>),
    /// No sidecar exists.
    Missing,
    /// Sidecar is readable but one of its compilation inputs changed.
    Stale(InvalidationReason),
    /// Sidecar was produced for an incompatible compiler or bytecode format.
    Incompatible(IncompatibilityReason),
    /// Sidecar bytes are malformed or use an unsupported envelope.
    Corrupt(FormatError),
}

pub(super) fn validate(unit: CompiledUnit, expected: &ExpectedUnitIdentity) -> SidecarLoad {
    let identity = &unit.identity;
    if identity.compiler_version != expected.compiler_version {
        return SidecarLoad::Incompatible(IncompatibilityReason::Compiler);
    }
    if identity.bytecode_version != expected.bytecode_version {
        return SidecarLoad::Incompatible(IncompatibilityReason::Bytecode);
    }
    let reason = if identity.unit_name != expected.unit_name {
        Some(InvalidationReason::UnitName)
    } else if identity.source_hash != expected.source_hash {
        Some(InvalidationReason::Source)
    } else if identity.options_hash != expected.options_hash {
        Some(InvalidationReason::Options)
    } else if !dependencies_match(&identity.dependencies, &expected.dependencies) {
        Some(InvalidationReason::Dependencies)
    } else {
        None
    };
    reason.map_or(SidecarLoad::Reusable(Box::new(unit)), SidecarLoad::Stale)
}

fn dependencies_match(actual: &[DependencyIdentity], expected: &[DependencyIdentity]) -> bool {
    actual == expected
}

//! Sidecar reuse and invalidation classification.

use crate::{CompiledUnit, DependencyIdentity, ExpectedUnitIdentity};

use super::SidecarCorruption;

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

/// Result of loading and validating a source-adjacent sidecar payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidecarStatus<T> {
    /// Sidecar matches every current input and may be reused.
    Reusable(Box<T>),
    /// No sidecar exists.
    Missing,
    /// Sidecar is readable but one of its compilation inputs changed.
    Stale(InvalidationReason),
    /// Sidecar was produced for an incompatible compiler or bytecode format.
    Incompatible(IncompatibilityReason),
    /// Sidecar envelope or semantic payloads are malformed.
    Corrupt(SidecarCorruption),
}

/// Stack-object sidecar load result retained for P10 cleanup of the legacy implementation.
pub type SidecarLoad = SidecarStatus<super::LoadedUnit>;

/// Register-object sidecar load result used by the production register backend.
pub type RegisterSidecarLoad = SidecarStatus<super::LoadedRegisterUnit>;

pub(super) fn validate_identity<T>(
    unit: CompiledUnit,
    expected: &ExpectedUnitIdentity,
) -> Result<CompiledUnit, SidecarStatus<T>> {
    let identity = &unit.identity;
    if identity.compiler_version != expected.compiler_version {
        return Err(SidecarStatus::Incompatible(IncompatibilityReason::Compiler));
    }
    if identity.bytecode_version != expected.bytecode_version {
        return Err(SidecarStatus::Incompatible(IncompatibilityReason::Bytecode));
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
    match reason {
        Some(reason) => Err(SidecarStatus::Stale(reason)),
        None => Ok(unit),
    }
}

fn dependencies_match(actual: &[DependencyIdentity], expected: &[DependencyIdentity]) -> bool {
    actual == expected
}

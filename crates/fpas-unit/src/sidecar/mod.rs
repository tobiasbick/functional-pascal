//! Source-adjacent `.fpascu` loading, validation, and coordinated replacement.

mod atomic;
mod payload;
mod validation;

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::interface::UnitInterface;
use crate::object::RelocatableObject;
use crate::{CompiledUnit, ExpectedUnitIdentity, FormatError};

pub use validation::{IncompatibilityReason, InvalidationReason, SidecarLoad, SidecarStatus};

/// A reusable compiled unit carrying a validated relocatable object payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedUnit {
    /// Original hashed compiled-unit envelope.
    pub compiled: CompiledUnit,
    /// Decoded canonical public semantic interface.
    pub interface: UnitInterface,
    /// Decoded and validated relocatable register object.
    pub object: RelocatableObject,
}

/// Why a hash-consistent sidecar still cannot represent a reusable logical unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidecarCorruption {
    /// Binary envelope is malformed or exceeds resource limits.
    Format(FormatError),
    /// Interface payload cannot be decoded within its resource limit.
    InterfacePayload,
    /// Object payload cannot be decoded or violates object invariants.
    ObjectPayload,
    /// Envelope and interface identify different units.
    InterfaceUnitName {
        /// Envelope unit name.
        envelope: String,
        /// Interface unit name.
        payload: String,
    },
    /// Envelope and relocatable object identify different owners.
    ObjectOwner {
        /// Envelope unit name.
        envelope: String,
        /// Object owner.
        payload: String,
    },
    /// Interface exports duplicate case-insensitive symbol identities.
    DuplicateSymbol(String),
    /// A qualified interface symbol is not owned by the interface unit.
    SymbolOwner {
        /// Short exported symbol name.
        symbol: String,
        /// Stored qualified symbol name.
        qualified_name: String,
        /// Interface unit name.
        unit_name: String,
    },
}

/// Returns the `.fpascu` sidecar path for a `.fpas` source.
#[must_use]
pub fn sidecar_path(source_path: &Path) -> PathBuf {
    source_path.with_extension("fpascu")
}

/// Loads and validates a source-adjacent compiled unit.
///
/// # Errors
///
/// Returns [`SidecarError`] when the coordination lock or filesystem cannot be accessed. Invalid
/// artifact contents are classified in [`SidecarLoad`].
pub fn load_sidecar(
    source_path: &Path,
    expected: &ExpectedUnitIdentity,
) -> Result<SidecarLoad, SidecarError> {
    load_sidecar_with(source_path, expected, payload::validate)
}

fn load_sidecar_with<T>(
    source_path: &Path,
    expected: &ExpectedUnitIdentity,
    validate_payload: fn(CompiledUnit) -> Result<T, SidecarCorruption>,
) -> Result<SidecarStatus<T>, SidecarError> {
    let path = sidecar_path(source_path);
    let _lock = atomic::acquire_read_lock(&path)?;
    let metadata = match std::fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(SidecarStatus::Missing);
        }
        Err(error) => {
            return Err(SidecarError::Io {
                operation: "inspect",
                path,
                error,
            });
        }
    };
    let file_size = usize::try_from(metadata.len()).unwrap_or(usize::MAX);
    if let Err(error) = crate::format::check_sidecar_size(file_size) {
        return Ok(SidecarStatus::Corrupt(SidecarCorruption::Format(error)));
    }
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(SidecarStatus::Missing);
        }
        Err(error) => {
            return Err(SidecarError::Io {
                operation: "read",
                path,
                error,
            });
        }
    };
    let unit = match crate::decode(&bytes) {
        Ok(unit) => unit,
        Err(FormatError::UnsupportedVersion(version)) => {
            return Ok(SidecarStatus::Incompatible(
                IncompatibilityReason::FormatVersion(version),
            ));
        }
        Err(error) => {
            return Ok(SidecarStatus::Corrupt(SidecarCorruption::Format(error)));
        }
    };
    let unit = match validation::validate_identity(unit, expected) {
        Ok(unit) => unit,
        Err(load) => return Ok(load),
    };
    Ok(match validate_payload(unit) {
        Ok(unit) => SidecarStatus::Reusable(Box::new(unit)),
        Err(error) => SidecarStatus::Corrupt(error),
    })
}

/// Atomically publishes one validated `.fpascu` beside its source.
pub fn write_sidecar(source_path: &Path, unit: &CompiledUnit) -> Result<PathBuf, SidecarError> {
    let sidecar = sidecar_path(source_path);
    let bytes = crate::encode(unit).map_err(SidecarError::Format)?;
    atomic::replace(&sidecar, &bytes)?;
    Ok(sidecar)
}

/// Failure to encode, read, or publish a compiled-unit sidecar.
#[derive(Debug)]
pub enum SidecarError {
    /// The in-memory object cannot be encoded.
    Format(FormatError),
    /// A filesystem operation failed.
    Io {
        /// Operation being attempted.
        operation: &'static str,
        /// Affected sidecar or temporary path.
        path: PathBuf,
        /// Underlying operating-system error.
        error: io::Error,
    },
    /// Another compiler invocation held the sidecar lock too long.
    LockTimeout(PathBuf),
}

impl fmt::Display for SidecarError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Format(error) => error.fmt(formatter),
            Self::Io {
                operation,
                path,
                error,
            } => write!(
                formatter,
                "failed to {operation} compiled unit `{}`: {error}",
                path.display()
            ),
            Self::LockTimeout(path) => write!(
                formatter,
                "timed out waiting for compiled unit `{}`; another compiler process may still be using it",
                path.display()
            ),
        }
    }
}

impl std::error::Error for SidecarError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Format(error) => Some(error),
            Self::Io { error, .. } => Some(error),
            Self::LockTimeout(_) => None,
        }
    }
}

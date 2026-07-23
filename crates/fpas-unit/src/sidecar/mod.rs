//! Source-adjacent `.fpascu` loading, validation, and coordinated replacement.

mod atomic;
mod validation;

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::{CompiledUnit, ExpectedUnitIdentity, FormatError};

pub use validation::{IncompatibilityReason, InvalidationReason, SidecarLoad};

/// Returns the `.fpascu` sidecar path for a `.fpas` source.
#[must_use]
pub fn sidecar_path(source_path: &Path) -> PathBuf {
    source_path.with_extension("fpascu")
}

/// Loads and validates a source-adjacent compiled unit.
pub fn load_sidecar(
    source_path: &Path,
    expected: &ExpectedUnitIdentity,
) -> Result<SidecarLoad, SidecarError> {
    let path = sidecar_path(source_path);
    atomic::wait_until_unlocked(&path)?;
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(SidecarLoad::Missing),
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
            return Ok(SidecarLoad::Incompatible(
                IncompatibilityReason::FormatVersion(version),
            ));
        }
        Err(error) => return Ok(SidecarLoad::Corrupt(error)),
    };
    Ok(validation::validate(unit, expected))
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
    /// Another compiler invocation held the sidecar write lock too long.
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
                "timed out waiting to replace compiled unit `{}`; another compiler process may still be writing it",
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

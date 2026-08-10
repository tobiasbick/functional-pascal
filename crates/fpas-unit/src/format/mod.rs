//! Deterministic bounded binary encoding for `.fpascu` files.

mod read;
mod write;

use std::fmt;

use crate::CompiledUnit;

pub use read::decode;
pub use write::encode;

pub(super) const MAGIC: &[u8; 8] = b"FPASCU\0\0";

/// Current `.fpascu` envelope format version.
pub const FORMAT_VERSION: u16 = 4;

/// Largest accepted encoded `.fpascu` file.
///
/// The budget covers both 64 MiB payloads plus 8 MiB for identity metadata.
pub const MAX_SIDECAR_BYTES: usize = 136 * 1024 * 1024;

pub(super) const MAX_STRING_BYTES: usize = 1024 * 1024;
pub(super) const MAX_DEPENDENCIES: usize = 65_535;
pub(crate) const MAX_PAYLOAD_BYTES: usize = 64 * 1024 * 1024;

/// Invalid or unsupported `.fpascu` binary data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    /// File header does not identify a Functional Pascal compiled unit.
    InvalidMagic,
    /// File uses an unsupported envelope version.
    UnsupportedVersion(u16),
    /// Required bytes are missing.
    Truncated(&'static str),
    /// A bounded field exceeds the format limit.
    LimitExceeded {
        /// Logical field name.
        field: &'static str,
        /// Encoded or requested size.
        size: usize,
        /// Largest accepted size.
        maximum: usize,
    },
    /// A string field is not valid UTF-8.
    InvalidUtf8(&'static str),
    /// Recorded interface hash does not match the interface payload.
    InterfaceHashMismatch,
    /// Recorded object hash does not match the implementation payload.
    ObjectHashMismatch,
    /// Extra bytes follow the complete object.
    TrailingBytes(usize),
}

impl fmt::Display for FormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(formatter, "invalid `.fpascu` magic header"),
            Self::UnsupportedVersion(version) => write!(
                formatter,
                "unsupported `.fpascu` format version {version}; expected {FORMAT_VERSION}"
            ),
            Self::Truncated(field) => {
                write!(formatter, "truncated `.fpascu` while reading {field}")
            }
            Self::LimitExceeded {
                field,
                size,
                maximum,
            } => write!(
                formatter,
                "`.fpascu` field `{field}` has size {size}, exceeding limit {maximum}"
            ),
            Self::InvalidUtf8(field) => {
                write!(formatter, "`.fpascu` field `{field}` is not valid UTF-8")
            }
            Self::InterfaceHashMismatch => {
                write!(
                    formatter,
                    "`.fpascu` interface hash does not match its payload"
                )
            }
            Self::ObjectHashMismatch => {
                write!(
                    formatter,
                    "`.fpascu` object hash does not match its payload"
                )
            }
            Self::TrailingBytes(count) => {
                write!(formatter, "`.fpascu` contains {count} trailing bytes")
            }
        }
    }
}

impl std::error::Error for FormatError {}

pub(super) fn validate_for_write(unit: &CompiledUnit) -> Result<(), FormatError> {
    check_size("unit_name", unit.identity.unit_name.len(), MAX_STRING_BYTES)?;
    check_size(
        "compiler_version",
        unit.identity.compiler_version.len(),
        MAX_STRING_BYTES,
    )?;
    check_size(
        "dependencies",
        unit.identity.dependencies.len(),
        MAX_DEPENDENCIES,
    )?;
    for dependency in &unit.identity.dependencies {
        check_size(
            "dependency.unit_name",
            dependency.unit_name.len(),
            MAX_STRING_BYTES,
        )?;
    }
    check_size("interface", unit.interface.len(), MAX_PAYLOAD_BYTES)?;
    check_size("object", unit.object.len(), MAX_PAYLOAD_BYTES)?;
    if unit.identity.interface_hash != crate::Digest::of(&unit.interface) {
        return Err(FormatError::InterfaceHashMismatch);
    }
    if unit.identity.object_hash != crate::Digest::of(&unit.object) {
        return Err(FormatError::ObjectHashMismatch);
    }
    check_size("file", encoded_size(unit)?, MAX_SIDECAR_BYTES)?;
    Ok(())
}

pub(crate) fn check_payload_size(field: &'static str, size: usize) -> Result<(), FormatError> {
    check_size(field, size, MAX_PAYLOAD_BYTES)
}

pub(crate) fn check_sidecar_size(size: usize) -> Result<(), FormatError> {
    check_size("file", size, MAX_SIDECAR_BYTES)
}

fn encoded_size(unit: &CompiledUnit) -> Result<usize, FormatError> {
    const FIXED_BYTES: usize = MAGIC.len()
        + size_of::<u16>()
        + size_of::<u32>()
        + 4 * crate::Digest::LENGTH
        + 5 * size_of::<u32>();
    let mut size = FIXED_BYTES;
    size = add_encoded_size(size, unit.identity.compiler_version.len())?;
    size = add_encoded_size(size, unit.identity.unit_name.len())?;
    for dependency in &unit.identity.dependencies {
        size = add_encoded_size(size, size_of::<u32>() + dependency.unit_name.len())?;
        size = add_encoded_size(size, crate::Digest::LENGTH)?;
    }
    size = add_encoded_size(size, unit.interface.len())?;
    add_encoded_size(size, unit.object.len())
}

fn add_encoded_size(size: usize, additional: usize) -> Result<usize, FormatError> {
    size.checked_add(additional)
        .ok_or(FormatError::LimitExceeded {
            field: "file",
            size: usize::MAX,
            maximum: MAX_SIDECAR_BYTES,
        })
}

pub(super) fn check_size(
    field: &'static str,
    size: usize,
    maximum: usize,
) -> Result<(), FormatError> {
    if size > maximum {
        return Err(FormatError::LimitExceeded {
            field,
            size,
            maximum,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{MAX_PAYLOAD_BYTES, MAX_SIDECAR_BYTES, check_payload_size, check_sidecar_size};

    #[test]
    fn payload_limit_accepts_exact_size_and_rejects_one_over() {
        assert!(check_payload_size("payload", MAX_PAYLOAD_BYTES).is_ok());
        assert!(check_payload_size("payload", MAX_PAYLOAD_BYTES + 1).is_err());
    }

    #[test]
    fn sidecar_limit_accepts_exact_size_and_rejects_one_over() {
        assert!(check_sidecar_size(MAX_SIDECAR_BYTES).is_ok());
        assert!(check_sidecar_size(MAX_SIDECAR_BYTES + 1).is_err());
    }
}

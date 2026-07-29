//! Deterministic bounded binary encoding for `.fpascp` files.

mod read;
mod write;

use std::fmt;

use crate::{ImageError, ProgramImage};

pub use read::decode;
pub use write::encode;

pub(super) const MAGIC: &[u8; 8] = b"FPASCP\0\0";

/// Current `.fpascp` envelope format version.
pub const PROGRAM_FORMAT_VERSION: u16 = 1;

pub(super) const MAX_STRING_BYTES: usize = 1024 * 1024;
pub(super) const MAX_UNITS: usize = 65_535;
pub(super) const MAX_SOURCE_PATHS: usize = 65_535;
pub(super) const MAX_PAYLOAD_BYTES: usize = 128 * 1024 * 1024;

/// Invalid or unsupported `.fpascp` binary data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    /// File header does not identify a Functional Pascal compiled program.
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
    /// Recorded payload hash does not match the executable payload.
    PayloadHashMismatch,
    /// Extra bytes follow the complete image.
    TrailingBytes(usize),
    /// The decoded or encoded in-memory image is invalid.
    Image(ImageError),
}

impl fmt::Display for FormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(formatter, "invalid `.fpascp` magic header"),
            Self::UnsupportedVersion(version) => write!(
                formatter,
                "unsupported `.fpascp` format version {version}; expected {PROGRAM_FORMAT_VERSION}"
            ),
            Self::Truncated(field) => {
                write!(formatter, "truncated `.fpascp` while reading {field}")
            }
            Self::LimitExceeded {
                field,
                size,
                maximum,
            } => write!(
                formatter,
                "`.fpascp` field `{field}` has size {size}, exceeding limit {maximum}"
            ),
            Self::InvalidUtf8(field) => {
                write!(formatter, "`.fpascp` field `{field}` is not valid UTF-8")
            }
            Self::PayloadHashMismatch => {
                write!(formatter, "`.fpascp` payload hash does not match its data")
            }
            Self::TrailingBytes(count) => {
                write!(formatter, "`.fpascp` contains {count} trailing bytes")
            }
            Self::Image(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for FormatError {}

impl From<ImageError> for FormatError {
    fn from(error: ImageError) -> Self {
        Self::Image(error)
    }
}

pub(super) fn validate_for_write(image: &ProgramImage) -> Result<(), FormatError> {
    image.validate()?;
    check_size(
        "compiler_version",
        image.identity().compiler_version.len(),
        MAX_STRING_BYTES,
    )?;
    check_size("units", image.identity().units.len(), MAX_UNITS)?;
    for unit in &image.identity().units {
        check_size("unit.unit_name", unit.unit_name.len(), MAX_STRING_BYTES)?;
    }
    check_size("source_paths", image.source_paths().len(), MAX_SOURCE_PATHS)?;
    for source_path in image.source_paths() {
        check_size("source_path", source_path.len(), MAX_STRING_BYTES)?;
    }
    Ok(())
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

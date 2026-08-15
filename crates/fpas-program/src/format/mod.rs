//! Deterministic bounded binary encoding for portable register `.fpascp` files.

mod debug;
mod debug_types;
mod executable;
mod header;
mod read;
mod sections;
mod write;

use std::fmt;

use crate::ImageError;

pub use read::decode;
pub use write::encode;

/// Current sectioned register `.fpascp` envelope format version.
pub const PROGRAM_FORMAT_VERSION: u16 = 14;

/// Invalid or unsupported `.fpascp` binary data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    /// File header does not identify a Functional Pascal compiled program.
    InvalidMagic,
    /// File uses an unsupported envelope version.
    UnsupportedVersion {
        /// Version recorded in the image.
        image: u16,
        /// Version accepted by this runtime.
        runtime: u16,
    },
    /// File uses an unsupported register-bytecode version.
    UnsupportedBytecodeVersion {
        /// Version recorded in the image.
        image: u32,
        /// Version accepted by this runtime.
        runtime: u32,
    },
    /// Reserved flags contain an assigned bit unknown to this decoder.
    UnsupportedFlags {
        /// Header or section-directory field containing the flags.
        field: &'static str,
        /// Unsupported encoded flags.
        flags: u16,
    },
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
    /// Extra bytes follow a complete image or section.
    TrailingBytes {
        /// Logical container containing extra bytes.
        container: &'static str,
        /// Number of unexplained bytes.
        count: usize,
    },
    /// The section directory has an invalid number of entries.
    SectionCount {
        /// Encoded section count.
        actual: usize,
        /// Required section count.
        expected: usize,
    },
    /// A section tag is unknown, duplicated, or out of canonical order.
    SectionTag {
        /// Directory position.
        index: usize,
        /// Encoded tag.
        actual: u16,
        /// Required tag at that position.
        expected: u16,
    },
    /// A section range is outside the payload or not contiguous with its predecessor.
    SectionRange {
        /// Section tag.
        tag: u16,
        /// Encoded section offset.
        offset: usize,
        /// Expected contiguous offset.
        expected_offset: usize,
        /// Encoded section length.
        length: usize,
        /// Total payload length.
        payload: usize,
    },
    /// A section item count does not fit its represented resource.
    SectionItemCount {
        /// Section tag.
        tag: u16,
        /// Encoded count.
        count: usize,
        /// Maximum accepted count.
        maximum: usize,
    },
    /// A fixed format field contains an unknown or noncanonical value.
    InvalidValue {
        /// Field containing the value.
        field: &'static str,
        /// Encoded value.
        value: u64,
    },
    /// The decoded or encoded in-memory image is invalid.
    Image(ImageError),
}

impl fmt::Display for FormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(formatter, "invalid `.fpascp` magic header"),
            Self::UnsupportedVersion { image, runtime } => write!(
                formatter,
                "unsupported `.fpascp` format version {image}; this runtime requires version {runtime}"
            ),
            Self::UnsupportedBytecodeVersion { image, runtime } => write!(
                formatter,
                "unsupported `.fpascp` bytecode version {image}; this runtime requires version {runtime}"
            ),
            Self::UnsupportedFlags { field, flags } => {
                write!(
                    formatter,
                    "unsupported `.fpascp` {field} flags 0x{flags:04x}"
                )
            }
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
            Self::TrailingBytes { container, count } => {
                write!(
                    formatter,
                    "`.fpascp` {container} contains {count} trailing bytes"
                )
            }
            Self::SectionCount { actual, expected } => write!(
                formatter,
                "`.fpascp` has {actual} executable sections; expected exactly {expected}"
            ),
            Self::SectionTag {
                index,
                actual,
                expected,
            } => write!(
                formatter,
                "`.fpascp` section directory entry {index} has tag {actual}; expected canonical tag {expected}"
            ),
            Self::SectionRange {
                tag,
                offset,
                expected_offset,
                length,
                payload,
            } => write!(
                formatter,
                "`.fpascp` section {tag} range offset {offset}, length {length} is not contiguous at {expected_offset} within payload length {payload}"
            ),
            Self::SectionItemCount {
                tag,
                count,
                maximum,
            } => write!(
                formatter,
                "`.fpascp` section {tag} declares {count} items, exceeding limit {maximum}"
            ),
            Self::InvalidValue { field, value } => {
                write!(
                    formatter,
                    "`.fpascp` field `{field}` has invalid value {value}"
                )
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

pub(super) fn checked_u32(field: &'static str, value: usize) -> Result<u32, FormatError> {
    u32::try_from(value).map_err(|_| FormatError::LimitExceeded {
        field,
        size: value,
        maximum: u32::MAX as usize,
    })
}

pub(super) fn check_limit(
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

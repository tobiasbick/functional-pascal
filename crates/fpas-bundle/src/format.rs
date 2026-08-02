//! Versioned footer format for an executable with an appended program image.

use std::fmt;

const MAGIC: &[u8; 8] = b"FPASAPP\0";
const FORMAT_VERSION: u16 = 1;
const FOOTER_LEN: usize = 24;
const MAX_NAME_BYTES: usize = 4096;

/// A validated program embedded in a host-native runner.
pub struct BundledProgram<'a> {
    /// Application name recorded by the packager.
    pub name: &'a str,
    /// Decoded and validated `.fpascp` program image.
    pub image: fpas_program::ProgramImage,
}

/// Invalid bundle input or executable footer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleError {
    /// The application name is empty.
    EmptyName,
    /// The application name exceeds the format limit.
    NameTooLong(usize),
    /// The application name is not valid UTF-8.
    InvalidName,
    /// The executable has no bundle footer.
    MissingFooter,
    /// The executable uses an unsupported bundle version.
    UnsupportedVersion(u16),
    /// The footer contains non-zero reserved data.
    ReservedData,
    /// The recorded lengths do not fit inside the executable.
    InvalidLengths,
    /// The embedded `.fpascp` is invalid.
    Program(String),
    /// The complete bundle exceeds addressable memory.
    BundleTooLarge,
}

impl fmt::Display for BundleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(formatter, "application name must not be empty"),
            Self::NameTooLong(size) => write!(
                formatter,
                "application name has {size} bytes, exceeding limit {MAX_NAME_BYTES}"
            ),
            Self::InvalidName => write!(formatter, "application name is not valid UTF-8"),
            Self::MissingFooter => write!(formatter, "executable has no FPAS application footer"),
            Self::UnsupportedVersion(version) => write!(
                formatter,
                "unsupported FPAS application format version {version}; expected {FORMAT_VERSION}"
            ),
            Self::ReservedData => write!(formatter, "FPAS application footer is malformed"),
            Self::InvalidLengths => {
                write!(formatter, "FPAS application payload lengths are invalid")
            }
            Self::Program(message) => write!(formatter, "embedded program is invalid: {message}"),
            Self::BundleTooLarge => write!(formatter, "FPAS application bundle is too large"),
        }
    }
}

impl std::error::Error for BundleError {}

/// Append a validated program image, application name, and footer to runner bytes.
///
/// # Errors
///
/// Returns an error for an invalid name or program image, or when the complete
/// bundle length exceeds the format's addressable limits.
pub fn encode(runner: &[u8], image: &[u8], name: &str) -> Result<Vec<u8>, BundleError> {
    validate_name(name)?;
    fpas_program::decode(image).map_err(|error| BundleError::Program(error.to_string()))?;
    let image_len = u64::try_from(image.len()).map_err(|_| BundleError::BundleTooLarge)?;
    let name_len = u32::try_from(name.len()).map_err(|_| BundleError::BundleTooLarge)?;
    let capacity = runner
        .len()
        .checked_add(image.len())
        .and_then(|size| size.checked_add(name.len()))
        .and_then(|size| size.checked_add(FOOTER_LEN))
        .ok_or(BundleError::BundleTooLarge)?;

    let mut bundled = Vec::with_capacity(capacity);
    bundled.extend_from_slice(runner);
    bundled.extend_from_slice(image);
    bundled.extend_from_slice(name.as_bytes());
    bundled.extend_from_slice(MAGIC);
    bundled.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    bundled.extend_from_slice(&0_u16.to_le_bytes());
    bundled.extend_from_slice(&image_len.to_le_bytes());
    bundled.extend_from_slice(&name_len.to_le_bytes());
    Ok(bundled)
}

/// Read and validate the program appended to a host-native executable.
///
/// # Errors
///
/// Returns an error when the footer, lengths, name, or embedded program image
/// violates the versioned bundle format.
pub fn decode(executable: &[u8]) -> Result<BundledProgram<'_>, BundleError> {
    let footer_start = executable
        .len()
        .checked_sub(FOOTER_LEN)
        .ok_or(BundleError::MissingFooter)?;
    let footer = &executable[footer_start..];
    if &footer[..8] != MAGIC {
        return Err(BundleError::MissingFooter);
    }
    let version = u16::from_le_bytes([footer[8], footer[9]]);
    if version != FORMAT_VERSION {
        return Err(BundleError::UnsupportedVersion(version));
    }
    if footer[10] != 0 || footer[11] != 0 {
        return Err(BundleError::ReservedData);
    }
    let image_len = usize::try_from(u64::from_le_bytes(
        footer[12..20]
            .try_into()
            .map_err(|_| BundleError::InvalidLengths)?,
    ))
    .map_err(|_| BundleError::InvalidLengths)?;
    let name_len = usize::try_from(u32::from_le_bytes(
        footer[20..24]
            .try_into()
            .map_err(|_| BundleError::InvalidLengths)?,
    ))
    .map_err(|_| BundleError::InvalidLengths)?;
    let payload_len = image_len
        .checked_add(name_len)
        .ok_or(BundleError::InvalidLengths)?;
    let payload_start = footer_start
        .checked_sub(payload_len)
        .ok_or(BundleError::InvalidLengths)?;
    let name_start = payload_start
        .checked_add(image_len)
        .ok_or(BundleError::InvalidLengths)?;
    let image_bytes = &executable[payload_start..name_start];
    let name = std::str::from_utf8(&executable[name_start..footer_start])
        .map_err(|_| BundleError::InvalidName)?;
    validate_name(name)?;
    let image = fpas_program::decode(image_bytes)
        .map_err(|error| BundleError::Program(error.to_string()))?;
    Ok(BundledProgram { name, image })
}

fn validate_name(name: &str) -> Result<(), BundleError> {
    if name.trim().is_empty() {
        return Err(BundleError::EmptyName);
    }
    if name.len() > MAX_NAME_BYTES {
        return Err(BundleError::NameTooLong(name.len()));
    }
    Ok(())
}

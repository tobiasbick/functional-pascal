//! Deterministic encoding of semantic interfaces.

use std::fmt;

use super::UnitInterface;

/// Invalid semantic-interface payload.
#[derive(Debug)]
pub enum InterfaceFormatError {
    /// Payload exceeds the compiled-unit resource limit.
    LimitExceeded {
        /// Encoded or requested size.
        size: usize,
        /// Largest accepted size.
        maximum: usize,
    },
    /// JSON serialization failed.
    Encode(serde_json::Error),
    /// JSON decoding failed or used an unknown shape/tag.
    Decode(serde_json::Error),
}

impl fmt::Display for InterfaceFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LimitExceeded { size, maximum } => write!(
                formatter,
                "unit interface payload has size {size}, exceeding limit {maximum}"
            ),
            Self::Encode(error) => write!(formatter, "cannot encode unit interface: {error}"),
            Self::Decode(error) => write!(formatter, "cannot decode unit interface: {error}"),
        }
    }
}

impl std::error::Error for InterfaceFormatError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LimitExceeded { .. } => None,
            Self::Encode(error) | Self::Decode(error) => Some(error),
        }
    }
}

/// Encode a canonical semantic interface.
///
/// # Errors
///
/// Returns [`InterfaceFormatError`] when serialization fails or the encoded
/// payload exceeds the compiled-unit payload limit.
pub fn encode_interface(interface: &UnitInterface) -> Result<Vec<u8>, InterfaceFormatError> {
    let bytes = serde_json::to_vec(&interface.clone().canonicalized())
        .map_err(InterfaceFormatError::Encode)?;
    check_size(bytes.len())?;
    Ok(bytes)
}

/// Decode and normalize a semantic interface.
///
/// # Errors
///
/// Returns [`InterfaceFormatError`] when the payload exceeds its resource
/// limit or is not a supported semantic-interface encoding.
pub fn decode_interface(bytes: &[u8]) -> Result<UnitInterface, InterfaceFormatError> {
    check_size(bytes.len())?;
    serde_json::from_slice::<UnitInterface>(bytes)
        .map(UnitInterface::canonicalized)
        .map_err(InterfaceFormatError::Decode)
}

fn check_size(size: usize) -> Result<(), InterfaceFormatError> {
    crate::format::check_payload_size("interface", size).map_err(|_| {
        InterfaceFormatError::LimitExceeded {
            size,
            maximum: crate::format::MAX_PAYLOAD_BYTES,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::check_size;
    use crate::format::MAX_PAYLOAD_BYTES;

    #[test]
    fn direct_interface_codec_enforces_payload_limit() {
        assert!(check_size(MAX_PAYLOAD_BYTES).is_ok());
        assert!(check_size(MAX_PAYLOAD_BYTES + 1).is_err());
    }
}

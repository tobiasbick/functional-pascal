//! Deterministic encoding of semantic interfaces.

use std::fmt;

use super::UnitInterface;

/// Invalid semantic-interface payload.
#[derive(Debug)]
pub enum InterfaceFormatError {
    /// JSON serialization failed.
    Encode(serde_json::Error),
    /// JSON decoding failed or used an unknown shape/tag.
    Decode(serde_json::Error),
}

impl fmt::Display for InterfaceFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encode(error) => write!(formatter, "cannot encode unit interface: {error}"),
            Self::Decode(error) => write!(formatter, "cannot decode unit interface: {error}"),
        }
    }
}

impl std::error::Error for InterfaceFormatError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Encode(error) | Self::Decode(error) => Some(error),
        }
    }
}

/// Encode a canonical semantic interface.
pub fn encode_interface(interface: &UnitInterface) -> Result<Vec<u8>, InterfaceFormatError> {
    serde_json::to_vec(&interface.clone().canonicalized()).map_err(InterfaceFormatError::Encode)
}

/// Decode and normalize a semantic interface.
pub fn decode_interface(bytes: &[u8]) -> Result<UnitInterface, InterfaceFormatError> {
    serde_json::from_slice::<UnitInterface>(bytes)
        .map(UnitInterface::canonicalized)
        .map_err(InterfaceFormatError::Decode)
}

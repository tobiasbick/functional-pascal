//! SDK-based serialization preserving an independent copy of the original wire value.

mod de;
mod finite;

use std::io::{self, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{CodecError, ErrorKind, RawJson};

/// Decoded typed payload alongside the original JSON, including fields the SDK ignores.
pub struct Decoded<T> {
    /// The pinned SDK's typed interpretation, including its documented defaults.
    pub value: T,
    /// Original validated JSON with unknown fields and number spellings intact.
    pub raw: RawJson,
}

/// Decode a pinned SDK type without replacing the original wire document.
pub fn decode<T: DeserializeOwned>(raw: RawJson) -> Result<Decoded<T>, CodecError> {
    let value = deserialize(&raw)?;
    Ok(Decoded { value, raw })
}

/// Deserialize a borrowed payload using the SDK's finite-number rules.
pub fn deserialize<T: DeserializeOwned>(raw: &RawJson) -> Result<T, CodecError> {
    let node = serde_json::from_str(raw.text()).map_err(CodecError::json)?;
    T::deserialize(de::Sdk(node)).map_err(CodecError::json)
}

/// Encode a pinned SDK type after validating finite numbers and the message budget.
pub fn encode<T: Serialize>(value: &T, maximum: usize) -> Result<RawJson, CodecError> {
    value.serialize(finite::Check::new())?;
    let mut output = Bounded {
        bytes: Vec::new(),
        maximum: maximum.min(crate::raw::MAX_RAW_BYTES),
        exceeded: false,
    };
    serde_json::to_writer(&mut output, value).map_err(|error| {
        if output.exceeded {
            CodecError::new(ErrorKind::LimitExceeded)
        } else {
            CodecError::json(error)
        }
    })?;
    let text =
        String::from_utf8(output.bytes).map_err(|_| CodecError::new(ErrorKind::InvalidMessage))?;
    RawJson::parse_bounded(&text, maximum)
}

struct Bounded {
    bytes: Vec<u8>,
    maximum: usize,
    exceeded: bool,
}

impl Write for Bounded {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.maximum.saturating_sub(self.bytes.len()) {
            self.exceeded = true;
            return Err(io::Error::other("ACP message limit exceeded"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

//! Typed payload conversion over the shared declaration catalog.
mod decode;
mod encode;
mod roots;
mod value;
mod wire;

use crate::{CodecError, ErrorKind, RawJson};
pub use fpas_acp_model::binding::*;
pub use roots::DecodedPayload;
pub use value::Value;

fn invalid() -> CodecError {
    CodecError::new(ErrorKind::InvalidMessage)
}

/// Statically dispatched wire conversions for a shared ACP declaration catalog.
pub trait CatalogCodec {
    /// Decode a named value while retaining SDK defaults and exact raw subtrees.
    fn decode(&self, key: &str, raw: &RawJson) -> Result<Value, CodecError>;
    /// Encode a named value after validation against the pinned SDK.
    fn encode(&self, key: &str, value: &Value) -> Result<RawJson, CodecError>;
    /// Test whether a root recognizes a method, including protocol extensions.
    fn supports_method(&self, root: &str, method: &str) -> bool;
    /// Decode a method-selected payload, retaining its original document.
    fn decode_message(
        &self,
        root: &str,
        method: &str,
        raw: RawJson,
    ) -> Result<DecodedPayload, CodecError>;
    /// Encode a root whose nominal family and method must agree.
    fn encode_message(
        &self,
        root: &str,
        method: &str,
        value: &Value,
    ) -> Result<RawJson, CodecError>;
}

impl CatalogCodec for Catalog {
    fn decode(&self, key: &str, raw: &RawJson) -> Result<Value, CodecError> {
        decode::decode(self, key, raw)
    }
    fn encode(&self, key: &str, value: &Value) -> Result<RawJson, CodecError> {
        encode::encode(self, key, value)
    }
    fn supports_method(&self, root: &str, method: &str) -> bool {
        roots::supports_method(self, root, method)
    }
    fn decode_message(
        &self,
        root: &str,
        method: &str,
        raw: RawJson,
    ) -> Result<DecodedPayload, CodecError> {
        roots::decode_message(self, root, method, raw)
    }
    fn encode_message(
        &self,
        root: &str,
        method: &str,
        value: &Value,
    ) -> Result<RawJson, CodecError> {
        roots::encode_message(self, root, method, value)
    }
}

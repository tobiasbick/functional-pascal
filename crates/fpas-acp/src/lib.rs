//! Bounded native connections, interaction state and MCP bridging for the pinned ACP SDK.
//!
//! Protocol types and codecs are re-exported from their independent crates.
pub mod connection;
pub mod interaction;
pub mod transport;

pub use fpas_acp_codec::{binding, codec};
pub use fpas_acp_model::{
    CodecError, ErrorKind, RawJson, SCHEMA_VERSION, UInt64, error, protocol, raw, schema, value,
};

/// Exact Rust SDK version used by this binding.
pub const SDK_VERSION: &str = "3.5.0";

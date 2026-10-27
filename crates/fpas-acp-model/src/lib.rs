//! Shared ACP values and SDK-derived declarations, independent of connections and MCP.
pub mod binding;
pub mod error;
pub mod raw;
pub mod schema;
pub mod value;

pub use agent_client_protocol_schema as protocol;
pub use error::{CodecError, ErrorKind};
pub use raw::RawJson;
pub use value::UInt64;

/// Exact schema version used to derive both language declarations and runtime codecs.
pub const SCHEMA_VERSION: &str = "1.12.0";

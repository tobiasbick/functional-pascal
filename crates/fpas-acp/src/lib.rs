//! Lossless ACP value and codec boundary for the pinned Rust SDK.
//!
//! This crate owns no VM values, compiler types, processes or executors.
//! Protocol serialization delegates to the SDK schema; arbitrary JSON stays raw.

pub mod binding;
pub mod codec;
pub mod error;
mod normalizers;
pub mod raw;
pub mod schema;
pub mod value;

pub use agent_client_protocol::schema as protocol;
pub use error::{CodecError, ErrorKind};
pub use raw::RawJson;
pub use value::UInt64;

/// Exact Rust SDK version used by this binding.
pub const SDK_VERSION: &str = "3.5.0";
/// Exact Rust schema crate version used by this binding.
pub const SCHEMA_VERSION: &str = "1.12.0";

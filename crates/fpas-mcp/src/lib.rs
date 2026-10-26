//! Official MCP SDK values, offline tool validation and bounded application dispatch.
//!
//! The native adapter owns no VM values and executes no application callbacks.
//! A serving task receives validated calls and returns their results explicitly.

pub mod catalog;
pub mod error;
pub mod limits;
pub mod runtime;
pub mod server;
pub mod transport;

pub use catalog::Catalog;
pub use error::Fault;
pub use limits::Limits;
pub use rmcp::model;
pub use runtime::{CleanupReceipt, HttpRuntime, Publication, PublishTicket};
pub use server::{Call, Endpoint, Reader, Responder, Server};
pub use transport::{HttpEndpoint, HttpError, HttpOptions};

/// Exact official Rust MCP SDK version used by this crate.
pub const SDK_VERSION: &str = "3.7.0";

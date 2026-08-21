//! Versioned UTF-8 JSON Lines debugger protocol.

pub(crate) mod encode;
pub(crate) mod encode_record;
pub(crate) mod parse;
pub(crate) mod protocol;
mod server;
mod transport;

pub use server::{JsonlServer, ServerStatus};
pub use transport::{serve, serve_script};

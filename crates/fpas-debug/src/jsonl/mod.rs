//! Versioned UTF-8 JSON Lines debugger protocol.

mod actor;
mod encode;
mod protocol;
mod server;
mod transport;

pub use server::{JsonlServer, ServerStatus};
pub use transport::{serve, serve_script};

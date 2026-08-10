//! Debug Adapter Protocol framing and request mapping.

mod framing;
mod server;

pub use framing::{read_message, serve, write_message};
pub use server::DapServer;

//! Owned message endpoints and bounded newline-delimited byte framing.

mod framing;
mod messages;

pub use framing::Framer;
pub use messages::{MessageEndpoint, MessageTransport, messages};
mod owned;
pub use owned::{ExitStatus, Frame, Transport, TransportStatus};

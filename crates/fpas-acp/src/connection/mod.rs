//! Bounded, Rust-owned connection state shared with hosted language adapters.

mod error;
mod limits;
pub(crate) mod queue;

pub use error::Fault;
pub use limits::Limits;
mod api;
mod budget;
mod client;
mod driver;
mod events;
mod host;
mod mcp;
mod replies;
mod runtime;
mod shared;
mod tools;
mod wire;
pub use api::{Connection, Reader};
pub use client::{Experimental, Settings, Snapshot};
pub use events::{Closed, Event, Lifecycle, Payload, RequestToken, Responder};
pub use host::HostClaim;
pub use runtime::Runtime;

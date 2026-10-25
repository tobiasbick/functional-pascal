//! Bounded, Rust-owned connection state shared with hosted language adapters.

mod error;
mod limits;
pub(crate) mod queue;

pub use error::Fault;
pub use limits::Limits;
mod api;
mod budget;
mod driver;
mod events;
mod runtime;
mod shared;
mod wire;
pub use api::{Connection, Reader};
pub use events::{Closed, Event, Lifecycle, Payload, RequestToken, Responder};
pub use runtime::Runtime;

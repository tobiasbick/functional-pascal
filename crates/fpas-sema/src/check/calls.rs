//! Shared call argument validation and generic inference.

mod arguments;
mod inference;
mod targets;

/// Metadata for checked callable-value invocations.
pub use targets::{ValueCallMap, ValueCallTarget};

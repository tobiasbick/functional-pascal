//! Links relocatable Functional Pascal objects into a verified executable.

mod emit;
mod error;
mod plan;

pub use emit::link_objects;
pub use error::LinkError;

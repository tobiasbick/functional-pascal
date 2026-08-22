#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::panic,
        clippy::unwrap_used,
        reason = "tests use explicit failures to keep fixture assertions focused"
    )
)]

//! Links relocatable Functional Pascal objects into a verified executable.

mod emit;
mod error;
mod plan;

pub use emit::link_objects;
pub use error::LinkError;

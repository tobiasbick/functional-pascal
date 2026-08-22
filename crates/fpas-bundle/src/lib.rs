#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::panic,
        clippy::unwrap_used,
        reason = "tests use explicit failures to keep fixture assertions focused"
    )
)]

//! Encoding and publication of host-native Functional Pascal applications.
//!
//! Documentation: `docs/pascal/program-structure/cli.md`.

mod format;
mod publication;

pub use format::{BundleError, BundledProgram, decode, encode};
pub use publication::publish;

//! Encoding and publication of host-native Functional Pascal applications.
//!
//! Documentation: `docs/pascal/program-structure/cli.md`.

mod format;
mod publication;

pub use format::{BundleError, BundledProgram, decode, encode};
pub use publication::publish;

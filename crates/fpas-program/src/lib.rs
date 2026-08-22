#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::panic,
        clippy::unwrap_used,
        reason = "tests use explicit failures to keep fixture assertions focused"
    )
)]

//! Persistent executable program images for Functional Pascal.
//!
//! Documentation: `docs/pascal/program-structure/projects.md`.

mod format;
mod identity;
mod image;

pub use format::{FormatError, PROGRAM_FORMAT_VERSION, decode, encode};
pub use identity::{Digest, LinkedUnitIdentity, ProgramIdentity};
pub use image::{ImageError, ProgramImage};

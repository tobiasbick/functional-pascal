//! Persistent executable program images for Functional Pascal.
//!
//! Documentation: `docs/pascal/program-structure/projects.md`.

mod format;
mod identity;
mod image;

pub use format::{FormatError, PROGRAM_FORMAT_VERSION, decode, encode};
pub use identity::{Digest, LinkedUnitIdentity, ProgramIdentity};
pub use image::{ImageError, ProgramImage};

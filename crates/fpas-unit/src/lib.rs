//! Compiled-unit identities, binary format, and `.fpascu` sidecar management.
//!
//! Documentation: `docs/pascal/program-structure/units.md` and
//! `docs/pascal/program-structure/projects.md`.

mod format;
mod identity;
pub mod interface;
pub mod object;
mod sidecar;

pub use format::{FORMAT_VERSION, FormatError, MAX_SIDECAR_BYTES, decode, encode};
pub use identity::{CompiledUnit, DependencyIdentity, Digest, ExpectedUnitIdentity, UnitIdentity};
pub use sidecar::{
    IncompatibilityReason, InvalidationReason, LoadedUnit, SidecarCorruption, SidecarError,
    SidecarLoad, SidecarStatus, load_sidecar, sidecar_path, write_sidecar,
};

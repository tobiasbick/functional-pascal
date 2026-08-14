//! Seeded initialization of a descendant below empty mutable debugger storage.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

mod diagnostics;
mod model;
mod prepare;

pub(in crate::vm::debug) use diagnostics::already_initialized;
pub use model::DebugStorageInitializationResult;
pub(in crate::vm::debug) use prepare::{
    format_target, live_root_is_empty, rebuild_root, reject_identity_bearing, require_empty_root,
    resolve_existing_path, validate_seed,
};

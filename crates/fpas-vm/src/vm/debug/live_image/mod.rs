//! Live-image compatibility classification and bounded commit preparation.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

mod class;
mod classify;
mod commit;
mod fingerprint;

pub use class::{LiveImageClassification, LiveImageReplaceResult, LiveImageUpdateClass};
pub use classify::classify_live_image;
pub(in crate::vm::debug) use commit::PreparedLiveImageCommit;

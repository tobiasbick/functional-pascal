//! Live-image compatibility classification without replacement.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

mod class;
mod classify;
mod fingerprint;

pub use class::{LiveImageClassification, LiveImageUpdateClass};
pub use classify::classify_live_image;

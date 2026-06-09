//! `Std.Test` runtime — assertion helpers and intrinsic dispatch.
//!
//! **Documentation:** `docs/pascal/std/test.md` (from the repository root).

mod assert;
mod intrinsic;
mod skip_state;

pub(crate) use intrinsic::run;
pub use skip_state::{reset_test_skip_state, test_was_skipped};

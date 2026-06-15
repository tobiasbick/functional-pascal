//! `Std.Test` runtime — assertion helpers and intrinsic dispatch.
//!
//! **Documentation:** `docs/pascal/std/test.md` (from the repository root).

mod assert;
mod intrinsic;
mod screen_assert;
mod skip_state;

pub(crate) use intrinsic::run;
pub use screen_assert::{assert_screen_cell, assert_screen_line, assert_view_rect};
pub use skip_state::{reset_test_skip_state, test_was_skipped};

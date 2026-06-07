//! `Std.Test` runtime — assertion helpers and intrinsic dispatch.
//!
//! **Documentation:** `docs/pascal/std/test.md` (from the repository root).

mod assert;
mod intrinsic;

pub(crate) use intrinsic::run;

//! `Std.Tui` intrinsic discriminants.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).
//!
//! Variant bodies live under [`variants/`](variants/) and are stitched by `build.rs`.

mod generated {
    include!(concat!(env!("OUT_DIR"), "/tui_intrinsic.rs"));
}

pub use generated::TuiIntrinsic;

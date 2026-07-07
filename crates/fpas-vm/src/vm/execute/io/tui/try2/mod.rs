//! TUI try-2 bridge (Rust-owned widget tree).
//!
//! New implementation for the rewrite on branch `refactor/tui-try-2`. Coexists with
//! try-1 modules until phase 7 deletion. Not wired to Pascal intrinsics until phase 2+.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-architecture.md`

pub mod app;
pub mod geometry;
pub mod headless;
pub mod intrinsics;
pub mod modals;
pub mod records;
pub mod registry;
pub mod session;
pub mod views;

pub(in crate::vm::execute::io::tui::try2) use modals::try2_exec_view;
#[allow(unused_imports, reason = "wired in phase 2 intrinsics")]
pub(in crate::vm::execute::io::tui::try2) use views::{
    try2_dialog_add_button, try2_dialog_new_modal,
};

pub(in crate::vm) use session::Try2Session;

//! TUI try-2 bridge (Rust-owned widget tree).
//!
//! Coexists with try-1 modules until phase 7 deletion. Phase-2 Pascal intrinsics
//! (`Dialog.NewModal`, `Button.New`, `Application.ExecView`, …) dispatch here.
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-architecture.md`

pub mod app;
pub mod events;
pub mod geometry;
pub mod headless;
pub mod intrinsics;
pub mod modals;
pub mod records;
pub mod registry;
pub mod run;
pub mod session;
pub mod testing;
pub mod views;

pub(in crate::vm::execute::io::tui::try2) use modals::try2_exec_view;
pub(in crate::vm::execute::io::tui) use run::try2_application_run;
#[allow(unused_imports, reason = "wired in phase 2 intrinsics")]
pub(in crate::vm::execute::io::tui::try2) use views::{
    try2_dialog_add_button, try2_dialog_new_modal,
};

pub(in crate::vm) use session::Try2Session;

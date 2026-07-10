//! TUI try-2 bridge (Rust-owned widget tree).
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-architecture.md`

pub mod app;
pub mod chrome;
pub mod events;
pub mod file_dialog;
pub mod headless;
pub mod intrinsics;
pub mod message_box;
pub mod modals;
pub mod records;
pub mod registry;
pub mod run;
pub mod session;
pub mod testing;
pub mod view_click;
pub mod view_lookup;
pub mod views;

pub(in crate::vm::execute::io::tui) use chrome::{try2_set_menu_bar, try2_set_status_line};
pub(in crate::vm::execute::io::tui) use file_dialog::try2_run_file_dialog;
pub(in crate::vm::execute::io::tui) use message_box::try2_message_box;
pub(in crate::vm::execute::io::tui) use run::{try2_application_run, try2_application_run_loop};
#[allow(unused_imports, reason = "wired in phase 2 intrinsics")]
pub(in crate::vm::execute::io::tui::try2) use views::{
    try2_dialog_add_button, try2_dialog_new_modal,
};

pub(in crate::vm) use session::Try2Session;

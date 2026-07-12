//! TUI Turbo Vision bridge (Rust-owned widget tree).
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

pub mod app;
pub(in crate::vm::execute::io::tui) mod bridged_check_box;
pub(in crate::vm::execute::io::tui) mod bridged_outline;
pub(in crate::vm::execute::io::tui) mod bridged_radio_button;
pub mod chrome;
mod chrome_input;
pub(in crate::vm::execute::io::tui) mod chrome_layout;
pub mod events;
pub mod file_dialog;
pub mod geometry;
pub(in crate::vm::execute::io::tui) mod handles;
pub mod headless;
pub(in crate::vm::execute::io::tui) mod headless_backend;
pub(in crate::vm::execute::io::tui) mod headless_draw;
pub(in crate::vm::execute::io::tui) mod input_events;
pub mod intrinsics;
pub mod message_box;
pub mod modals;
pub mod records;
pub mod registry;
pub mod run;
pub mod session;
pub mod testing;
pub(in crate::vm::execute::io::tui) mod testing_lifecycle;
pub mod view_click;
pub mod view_lookup;
pub mod views;

pub(in crate::vm::execute::io::tui) use chrome::{bridge_set_menu_bar, bridge_set_status_line};
pub(in crate::vm::execute::io::tui) use run::{
    bridge_application_run, bridge_application_run_loop,
};
#[cfg(test)]
#[allow(unused_imports, reason = "test-only dialog button helper")]
pub(in crate::vm::execute::io::tui::bridge) use views::{
    bridge_dialog_add_button, bridge_dialog_new_modal,
};

pub(in crate::vm) use session::TurboVisionSession;
mod application_intrinsics;
mod application_records;
mod commands;
pub(in crate::vm::execute::io::tui) mod handle_records;
pub(in crate::vm::execute::io::tui) mod lifecycle;
pub(in crate::vm::execute::io::tui) mod outline_nodes;
pub(in crate::vm::execute::io::tui) mod session_app;

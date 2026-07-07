//! Try-2 view construction (direct turbo-vision ownership).
//!
//! **Documentation:** `docs/refactor-tui-try-2/upstream-mapping.md`

mod button;
mod dialog;

pub(in crate::vm::execute::io::tui::try2) use button::try2_dialog_add_button;
pub(in crate::vm::execute::io::tui::try2) use dialog::try2_dialog_new_modal;

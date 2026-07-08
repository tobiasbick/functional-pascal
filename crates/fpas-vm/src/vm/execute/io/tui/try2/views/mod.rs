//! Try-2 view construction (direct turbo-vision ownership).
//!
//! **Documentation:** `docs/refactor-tui-try-2/upstream-mapping.md`

mod attach;
mod button;
mod check_box;
mod desktop;
mod dialog;
mod input_line;
mod static_text;
mod window;

pub(in crate::vm::execute::io::tui::try2) use attach::{
    try2_dialog_attach_child, try2_window_attach_child,
};
pub(in crate::vm::execute::io::tui::try2) use button::{try2_button_new, try2_dialog_add_button};
pub(in crate::vm::execute::io::tui::try2) use check_box::{
    try2_check_box_checked, try2_check_box_new, try2_check_box_set_checked,
};
pub(in crate::vm::execute::io::tui::try2) use desktop::try2_desktop_add;
pub(in crate::vm::execute::io::tui::try2) use dialog::try2_dialog_new_modal;
pub(in crate::vm::execute::io::tui::try2) use input_line::{
    try2_input_line_new, try2_input_line_set_text, try2_input_line_text,
};
pub(in crate::vm::execute::io::tui::try2) use static_text::try2_static_text_new;
pub(in crate::vm::execute::io::tui::try2) use window::try2_window_new;

//! Turbo Vision bridge view construction (direct turbo-vision ownership).
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

mod attach;
mod button;
mod check_box;
mod desktop;
mod dialog;
mod editor_window;
mod input_line;
mod list_box;
mod memo;
mod outline;
mod radio_button;
mod static_text;
mod text_viewer;
mod window;

pub(in crate::vm::execute::io::tui::bridge) use attach::{
    bridge_dialog_attach_child, bridge_window_attach_child,
};
#[cfg(test)]
pub(in crate::vm::execute::io::tui::bridge) use button::bridge_dialog_add_button;
pub(in crate::vm::execute::io::tui::bridge) use button::{
    bridge_button_new, bridge_button_set_text,
};
pub(in crate::vm::execute::io::tui::bridge) use check_box::{
    bridge_check_box_checked, bridge_check_box_new, bridge_check_box_set_checked,
};
pub(in crate::vm::execute::io::tui::bridge) use desktop::bridge_desktop_add;
pub(in crate::vm::execute::io::tui::bridge) use dialog::{
    bridge_dialog_new_modal, bridge_dialog_set_title,
};
pub(in crate::vm::execute::io::tui::bridge) use editor_window::bridge_editor_window_new;
pub(in crate::vm::execute::io::tui::bridge) use input_line::{
    bridge_input_line_new, bridge_input_line_set_text, bridge_input_line_text,
};
pub(in crate::vm::execute::io::tui::bridge) use list_box::{
    bridge_list_box_new, bridge_list_box_selection, bridge_list_box_set_items,
};
pub(in crate::vm::execute::io::tui::bridge) use memo::{bridge_memo_new, bridge_memo_set_text};
pub(in crate::vm::execute::io::tui::bridge) use outline::{
    bridge_outline_new, bridge_outline_selected_text, bridge_outline_selection,
    bridge_outline_set_nodes,
};
pub(in crate::vm::execute::io::tui::bridge) use radio_button::{
    bridge_radio_button_new, bridge_radio_button_selected, bridge_radio_button_set_selected,
};
pub(in crate::vm::execute::io::tui::bridge) use static_text::{
    bridge_static_text_new, bridge_static_text_set_text,
};
pub(in crate::vm::execute::io::tui::bridge) use text_viewer::{
    bridge_text_viewer_new, bridge_text_viewer_set_text,
};
pub(in crate::vm::execute::io::tui::bridge) use window::{
    bridge_window_new, bridge_window_set_title,
};

//! Turbo Vision view snapshots and upstream widget construction.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use super::command_map::fpas_command_to_turbo_vision;
use super::tv_geometry::turbo_rect;
use crate::vm::shared::{
    TurboVisionButton, TurboVisionCheckBox, TurboVisionInputLine, TurboVisionListBox,
    TurboVisionMemo, TurboVisionObject, TurboVisionRadioButton, TurboVisionRect,
    TurboVisionStaticText, TurboVisionStatusItem, TurboVisionTextViewer,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use turbo_vision::views::{
    dialog::Dialog, input_line::InputLine, listbox::ListBox, status_line::StatusItem,
    status_line::StatusLine, window::Window,
};

pub(in crate::vm::execute::io::tui) struct TurboVisionWindowSnapshot {
    pub bounds: TurboVisionRect,
    pub title: String,
    pub children: Vec<TurboVisionChildSnapshot>,
}

pub(in crate::vm::execute::io::tui) struct TurboVisionDialogSnapshot {
    pub bounds: TurboVisionRect,
    pub title: String,
    pub children: Vec<TurboVisionChildSnapshot>,
}

pub(in crate::vm::execute::io::tui) struct TurboVisionMenuBarSnapshot {
    pub bounds: TurboVisionRect,
    pub menus: Vec<crate::vm::shared::TurboVisionMenu>,
}

pub(in crate::vm::execute::io::tui) struct TurboVisionStatusLineSnapshot {
    pub bounds: TurboVisionRect,
    pub items: Vec<TurboVisionStatusItem>,
}

pub(in crate::vm::execute::io::tui) enum TurboVisionChildSnapshot {
    Button(TurboVisionButton),
    StaticText(TurboVisionStaticText),
    Memo(TurboVisionMemo),
    TextViewer(TurboVisionTextViewer),
    InputLine(TurboVisionInputLine),
    ListBox(TurboVisionListBox),
    CheckBox(TurboVisionCheckBox),
    RadioButton(TurboVisionRadioButton),
}

pub(in crate::vm::execute::io::tui) fn child_snapshots(
    objects: &std::collections::HashMap<u32, TurboVisionObject>,
    handles: &[u32],
) -> Vec<TurboVisionChildSnapshot> {
    handles
        .iter()
        .filter_map(|handle| match objects.get(handle) {
            Some(TurboVisionObject::Button(button)) => {
                Some(TurboVisionChildSnapshot::Button(button.clone()))
            }
            Some(TurboVisionObject::StaticText(static_text)) => {
                Some(TurboVisionChildSnapshot::StaticText(static_text.clone()))
            }
            Some(TurboVisionObject::Memo(memo)) => {
                Some(TurboVisionChildSnapshot::Memo(memo.clone()))
            }
            Some(TurboVisionObject::TextViewer(text_viewer)) => {
                Some(TurboVisionChildSnapshot::TextViewer(text_viewer.clone()))
            }
            Some(TurboVisionObject::InputLine(input_line)) => {
                Some(TurboVisionChildSnapshot::InputLine(input_line.clone()))
            }
            Some(TurboVisionObject::ListBox(list_box)) => {
                Some(TurboVisionChildSnapshot::ListBox(list_box.clone()))
            }
            Some(TurboVisionObject::CheckBox(check_box)) => {
                Some(TurboVisionChildSnapshot::CheckBox(check_box.clone()))
            }
            Some(TurboVisionObject::RadioButton(radio_button)) => {
                Some(TurboVisionChildSnapshot::RadioButton(radio_button.clone()))
            }
            _ => None,
        })
        .collect()
}

pub(in crate::vm::execute::io::tui) fn add_window_child(
    window: &mut Window,
    child: TurboVisionChildSnapshot,
    radio_groups: &HashMap<u16, Vec<crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell>>,
    tree_dirty: crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell,
) -> (
    turbo_vision::views::view::ViewId,
    Option<Rc<RefCell<String>>>,
) {
    match child {
        TurboVisionChildSnapshot::Button(button) => (
            window.add(Box::new(super::bridged_button::BridgedButton::new(
                turbo_rect(button.bounds),
                &button.text,
                fpas_command_to_turbo_vision(button.command_id),
                false,
            ))),
            None,
        ),
        TurboVisionChildSnapshot::StaticText(static_text) => (
            window.add(Box::new(
                super::bridged_static_text::BridgedStaticText::new(
                    turbo_rect(static_text.bounds),
                    &static_text.text,
                ),
            )),
            None,
        ),
        TurboVisionChildSnapshot::Memo(memo) => (
            window.add(Box::new(super::bridged_memo::BridgedMemo::new(
                turbo_rect(memo.bounds),
                &memo.text,
            ))),
            None,
        ),
        TurboVisionChildSnapshot::TextViewer(text_viewer) => (
            window.add(Box::new(
                super::bridged_text_viewer::BridgedTextViewer::new(
                    turbo_rect(text_viewer.bounds),
                    &text_viewer.text,
                ),
            )),
            None,
        ),
        TurboVisionChildSnapshot::InputLine(input_line) => {
            let binding = input_line.text_cell.view_binding();
            let view_id = window.add(Box::new(InputLine::new(
                turbo_rect(input_line.bounds),
                input_line.max_length,
                Rc::clone(&binding),
            )));
            (view_id, Some(binding))
        }
        TurboVisionChildSnapshot::ListBox(list_box) => {
            (window.add(Box::new(build_list_box(list_box))), None)
        }
        TurboVisionChildSnapshot::CheckBox(check_box) => (
            window.add(Box::new(super::bridged_check_box::BridgedCheckBox::new(
                turbo_rect(check_box.bounds),
                &check_box.text,
                check_box.checked_cell.clone(),
            ))),
            None,
        ),
        TurboVisionChildSnapshot::RadioButton(radio_button) => {
            let group_cells = radio_groups
                .get(&radio_button.group_id)
                .cloned()
                .unwrap_or_default();
            (
                window.add(Box::new(
                    super::bridged_radio_button::BridgedRadioButton::new(
                        turbo_rect(radio_button.bounds),
                        &radio_button.text,
                        radio_button.group_id,
                        radio_button.selected_cell.clone(),
                        group_cells,
                        tree_dirty,
                    ),
                )),
                None,
            )
        }
    }
}

pub(in crate::vm::execute::io::tui) fn add_dialog_child(
    dialog: &mut Dialog,
    child: TurboVisionChildSnapshot,
    child_handle: u32,
    input_bindings: &mut Vec<(u32, Rc<RefCell<String>>)>,
    radio_groups: &HashMap<u16, Vec<crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell>>,
    tree_dirty: crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell,
) -> (
    turbo_vision::views::view::ViewId,
    Option<Rc<RefCell<String>>>,
) {
    match child {
        TurboVisionChildSnapshot::Button(button) => (
            dialog.add(Box::new(super::bridged_button::BridgedButton::new(
                turbo_rect(button.bounds),
                &button.text,
                fpas_command_to_turbo_vision(button.command_id),
                false,
            ))),
            None,
        ),
        TurboVisionChildSnapshot::StaticText(static_text) => (
            dialog.add(Box::new(
                super::bridged_static_text::BridgedStaticText::new(
                    turbo_rect(static_text.bounds),
                    &static_text.text,
                ),
            )),
            None,
        ),
        TurboVisionChildSnapshot::Memo(memo) => (
            dialog.add(Box::new(super::bridged_memo::BridgedMemo::new(
                turbo_rect(memo.bounds),
                &memo.text,
            ))),
            None,
        ),
        TurboVisionChildSnapshot::TextViewer(text_viewer) => (
            dialog.add(Box::new(
                super::bridged_text_viewer::BridgedTextViewer::new(
                    turbo_rect(text_viewer.bounds),
                    &text_viewer.text,
                ),
            )),
            None,
        ),
        TurboVisionChildSnapshot::InputLine(input_line) => {
            let binding = input_line.text_cell.view_binding();
            input_bindings.push((child_handle, Rc::clone(&binding)));
            let view_id = dialog.add(Box::new(InputLine::new(
                turbo_rect(input_line.bounds),
                input_line.max_length,
                binding.clone(),
            )));
            (view_id, Some(binding))
        }
        TurboVisionChildSnapshot::ListBox(list_box) => (
            dialog.add(Box::new(super::bridged_list_box::BridgedListBox::new(
                turbo_rect(list_box.bounds),
                list_box.items,
                fpas_command_to_turbo_vision(list_box.command_id),
                list_box.selection_cell,
            ))),
            None,
        ),
        TurboVisionChildSnapshot::CheckBox(check_box) => (
            dialog.add(Box::new(super::bridged_check_box::BridgedCheckBox::new(
                turbo_rect(check_box.bounds),
                &check_box.text,
                check_box.checked_cell.clone(),
            ))),
            None,
        ),
        TurboVisionChildSnapshot::RadioButton(radio_button) => {
            let group_cells = radio_groups
                .get(&radio_button.group_id)
                .cloned()
                .unwrap_or_default();
            (
                dialog.add(Box::new(
                    super::bridged_radio_button::BridgedRadioButton::new(
                        turbo_rect(radio_button.bounds),
                        &radio_button.text,
                        radio_button.group_id,
                        radio_button.selected_cell.clone(),
                        group_cells,
                        tree_dirty,
                    ),
                )),
                None,
            )
        }
    }
}

pub(in crate::vm::execute::io::tui) fn radio_groups_from_snapshots(
    children: &[TurboVisionChildSnapshot],
) -> HashMap<u16, Vec<crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell>> {
    let mut groups = HashMap::new();
    for child in children {
        let TurboVisionChildSnapshot::RadioButton(radio_button) = child else {
            continue;
        };
        groups
            .entry(radio_button.group_id)
            .or_insert_with(Vec::new)
            .push(radio_button.selected_cell.clone());
    }
    groups
}

pub(in crate::vm::execute::io::tui) fn build_status_line(
    snapshot: TurboVisionStatusLineSnapshot,
) -> StatusLine {
    StatusLine::new(
        turbo_rect(snapshot.bounds),
        snapshot
            .items
            .into_iter()
            .map(|item| {
                StatusItem::new(
                    &item.text,
                    item.key_code,
                    fpas_command_to_turbo_vision(item.command_id),
                )
            })
            .collect(),
    )
}

/// Build a modal Turbo Vision dialog view from a live FPAS dialog handle.
pub(in crate::vm::execute::io::tui) fn turbo_vision_build_modal_dialog(
    objects: &HashMap<u32, crate::vm::shared::TurboVisionObject>,
    handle: u32,
    input_bindings: &mut Vec<(u32, Rc<RefCell<String>>)>,
    tree_dirty: crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell,
) -> Option<Box<Dialog>> {
    let crate::vm::shared::TurboVisionObject::Dialog(dialog) = objects.get(&handle)? else {
        return None;
    };
    let mut dialog_view = Dialog::new_modal(turbo_rect(dialog.bounds), &dialog.title);
    let radio_groups = radio_groups(objects, &dialog.children);
    for child_handle in &dialog.children {
        let Some(child) = child_snapshot(objects, *child_handle) else {
            continue;
        };
        let _ = add_dialog_child(
            &mut dialog_view,
            child,
            *child_handle,
            input_bindings,
            &radio_groups,
            tree_dirty.clone(),
        );
    }
    Some(dialog_view)
}

fn build_list_box(snapshot: TurboVisionListBox) -> ListBox {
    let mut list_box = ListBox::new(
        turbo_rect(snapshot.bounds),
        fpas_command_to_turbo_vision(snapshot.command_id),
    );
    list_box.set_items(snapshot.items);
    list_box
}

fn radio_groups(
    objects: &HashMap<u32, crate::vm::shared::TurboVisionObject>,
    handles: &[u32],
) -> HashMap<u16, Vec<crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell>> {
    let mut groups = HashMap::new();
    for handle in handles {
        let Some(TurboVisionObject::RadioButton(radio_button)) = objects.get(handle) else {
            continue;
        };
        groups
            .entry(radio_button.group_id)
            .or_insert_with(Vec::new)
            .push(radio_button.selected_cell.clone());
    }
    groups
}

fn child_snapshot(
    objects: &HashMap<u32, crate::vm::shared::TurboVisionObject>,
    handle: u32,
) -> Option<TurboVisionChildSnapshot> {
    match objects.get(&handle) {
        Some(TurboVisionObject::Button(button)) => {
            Some(TurboVisionChildSnapshot::Button(button.clone()))
        }
        Some(TurboVisionObject::StaticText(static_text)) => {
            Some(TurboVisionChildSnapshot::StaticText(static_text.clone()))
        }
        Some(TurboVisionObject::Memo(memo)) => Some(TurboVisionChildSnapshot::Memo(memo.clone())),
        Some(TurboVisionObject::TextViewer(text_viewer)) => {
            Some(TurboVisionChildSnapshot::TextViewer(text_viewer.clone()))
        }
        Some(TurboVisionObject::InputLine(input_line)) => {
            Some(TurboVisionChildSnapshot::InputLine(input_line.clone()))
        }
        Some(TurboVisionObject::ListBox(list_box)) => {
            Some(TurboVisionChildSnapshot::ListBox(list_box.clone()))
        }
        Some(TurboVisionObject::CheckBox(check_box)) => {
            Some(TurboVisionChildSnapshot::CheckBox(check_box.clone()))
        }
        Some(TurboVisionObject::RadioButton(radio_button)) => {
            Some(TurboVisionChildSnapshot::RadioButton(radio_button.clone()))
        }
        _ => None,
    }
}

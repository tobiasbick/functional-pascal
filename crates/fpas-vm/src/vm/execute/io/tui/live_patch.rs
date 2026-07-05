//! Incremental live turbo-vision view updates for FPAS data mutations.
//!
//! Avoids a full desktop rebuild when upstream setters can mirror FPAS handle state.
//!
//! **Documentation:** `docs/refactor/tui-bridge/05-reduce-reconcile-rebuild.md`

use super::bridged_check_box::BridgedCheckBox;
use super::bridged_list_box::BridgedListBox;
use super::bridged_radio_button::BridgedRadioButton;
use crate::vm::Worker;
use crate::vm::shared::TurboVisionObject;
use std::collections::HashMap;
use turbo_vision::core::geometry::Point;
use turbo_vision::views::desktop::Desktop;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::listbox::ListBox;
use turbo_vision::views::view::{View, ViewId};
use turbo_vision::views::window::Window;

/// Data mutation that may be applied to an existing live turbo-vision tree.
pub(in crate::vm::execute::io::tui) enum LiveDataMutation {
    SetTitle {
        handle: u32,
        title: String,
    },
    SetChecked {
        handle: u32,
    },
    SetListItems {
        handle: u32,
        items: Vec<String>,
        selection: Option<usize>,
    },
}

enum LiveViewLocation {
    DesktopRoot(ViewId),
    DesktopChild { root_index: usize, view_id: ViewId },
}

impl Worker {
    /// After a FPAS data mutation, patch the live session or fall back to structural reconcile.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_after_data_mutation(
        &mut self,
        mutation: LiveDataMutation,
    ) {
        if self.with_tui(|tui| tui.session.is_headless()) {
            if self.turbo_vision_try_headless_data_mutation(&mutation) {
                self.mark_turbo_vision_headless_repaint();
            } else {
                self.mark_turbo_vision_tree_dirty();
            }
            return;
        }
        if !self.turbo_vision_try_live_data_mutation(mutation) {
            self.mark_turbo_vision_tree_dirty();
        }
    }

    fn turbo_vision_try_headless_data_mutation(&mut self, mutation: &LiveDataMutation) -> bool {
        let view_ids = self.with_tui(|tui| tui.turbo_vision.live_view_ids.clone());
        let child_desktop_indices =
            self.with_tui(|tui| tui.turbo_vision.live_child_desktop_indices.clone());
        let mut app_slot = self.headless_tv_app.take();
        let Some(app) = app_slot.as_mut() else {
            self.headless_tv_app = app_slot;
            return false;
        };
        let ok = apply_live_data_mutation_to_desktop(
            app.desktop_mut(),
            &view_ids,
            &child_desktop_indices,
            mutation,
            self,
        );
        self.headless_tv_app = app_slot;
        ok
    }

    fn turbo_vision_try_live_data_mutation(&mut self, mutation: LiveDataMutation) -> bool {
        if !self.turbo_vision_live_app_active() {
            return false;
        }
        let view_ids = self.with_tui(|tui| tui.turbo_vision.live_view_ids.clone());
        let child_desktop_indices =
            self.with_tui(|tui| tui.turbo_vision.live_child_desktop_indices.clone());
        let mut app = match self.live_turbo_vision_app.take() {
            Some(app) => app,
            None => return false,
        };
        let ok = apply_live_data_mutation_to_desktop(
            &mut app.desktop,
            &view_ids,
            &child_desktop_indices,
            &mutation,
            self,
        );
        self.live_turbo_vision_app = Some(app);
        ok
    }

    pub(in crate::vm::execute::io::tui) fn turbo_vision_register_live_view_id(
        &self,
        handle: u32,
        view_id: ViewId,
    ) {
        self.with_tui(|tui| {
            tui.turbo_vision
                .live_view_ids
                .insert(handle, view_id.as_u16());
        });
    }

    pub(in crate::vm::execute::io::tui) fn turbo_vision_register_live_child_view(
        &self,
        handle: u32,
        desktop_root_index: usize,
        view_id: ViewId,
    ) {
        self.with_tui(|tui| {
            tui.turbo_vision
                .live_view_ids
                .insert(handle, view_id.as_u16());
            tui.turbo_vision
                .live_child_desktop_indices
                .insert(handle, desktop_root_index);
        });
    }

    pub(in crate::vm::execute::io::tui) fn turbo_vision_clear_live_view_ids(&self) {
        self.with_tui(|tui| {
            tui.turbo_vision.live_view_ids.clear();
            tui.turbo_vision.live_child_desktop_indices.clear();
        });
    }

    /// Top-left desktop coordinate for a registered live view (headless mouse routing).
    pub(in crate::vm::execute::io::tui) fn turbo_vision_live_view_click_point(
        &mut self,
        handle: u32,
    ) -> Option<Point> {
        let view_ids = self.with_tui(|tui| tui.turbo_vision.live_view_ids.clone());
        let child_desktop_indices =
            self.with_tui(|tui| tui.turbo_vision.live_child_desktop_indices.clone());
        let mut app_slot = self.headless_tv_app.take();
        let point = app_slot.as_mut().and_then(|app| {
            live_view_bounds_origin(app.desktop_mut(), &view_ids, &child_desktop_indices, handle)
        });
        self.headless_tv_app = app_slot;
        point
    }
}

fn apply_live_data_mutation_to_desktop(
    desktop: &mut turbo_vision::views::desktop::Desktop,
    view_ids: &HashMap<u32, u16>,
    child_desktop_indices: &HashMap<u32, usize>,
    mutation: &LiveDataMutation,
    worker: &Worker,
) -> bool {
    match mutation {
        LiveDataMutation::SetTitle { handle, title } => {
            let Some(location) = locate_live_view(view_ids, child_desktop_indices, *handle) else {
                return false;
            };
            let title = title.clone();
            with_live_view_mut(desktop, location, |view| {
                if let Some(window) = view.as_any_mut().downcast_mut::<Window>() {
                    window.set_title(&title);
                    return true;
                }
                if let Some(dialog) = view.as_any_mut().downcast_mut::<Dialog>() {
                    dialog.set_title(&title);
                    return true;
                }
                false
            })
        }
        LiveDataMutation::SetChecked { handle } => {
            let group_handles = worker.turbo_vision_radio_group_handles(*handle);
            let targets = if group_handles.is_empty() {
                vec![*handle]
            } else {
                group_handles
            };
            targets.iter().all(|&member| {
                patch_checked_handle(desktop, view_ids, child_desktop_indices, member)
            })
        }
        LiveDataMutation::SetListItems {
            handle,
            items,
            selection,
        } => patch_list_items(
            desktop,
            view_ids,
            child_desktop_indices,
            *handle,
            items.clone(),
            *selection,
        ),
    }
}

fn view_id_for_handle(view_ids: &HashMap<u32, u16>, handle: u32) -> Option<ViewId> {
    view_ids.get(&handle).copied().map(ViewId::from_u16)
}

fn locate_live_view(
    view_ids: &HashMap<u32, u16>,
    child_desktop_indices: &HashMap<u32, usize>,
    handle: u32,
) -> Option<LiveViewLocation> {
    let view_id = view_id_for_handle(view_ids, handle)?;
    if let Some(root_index) = child_desktop_indices.get(&handle) {
        return Some(LiveViewLocation::DesktopChild {
            root_index: *root_index,
            view_id,
        });
    }
    Some(LiveViewLocation::DesktopRoot(view_id))
}

fn with_live_view_mut(
    desktop: &mut Desktop,
    location: LiveViewLocation,
    patch: impl FnOnce(&mut dyn View) -> bool,
) -> bool {
    match location {
        LiveViewLocation::DesktopRoot(view_id) => {
            let Some(view) = desktop.child_by_id_mut(view_id) else {
                return false;
            };
            patch(view)
        }
        LiveViewLocation::DesktopChild {
            root_index,
            view_id,
        } => {
            let root = desktop.child_at_mut(root_index);
            if let Some(window) = root.as_any_mut().downcast_mut::<Window>() {
                let Some(child) = window.child_by_id_mut(view_id) else {
                    return false;
                };
                return patch(child);
            }
            if let Some(dialog) = root.as_any_mut().downcast_mut::<Dialog>() {
                let Some(child) = dialog.child_by_id_mut(view_id) else {
                    return false;
                };
                return patch(child);
            }
            false
        }
    }
}

fn live_view_bounds_origin(
    desktop: &mut Desktop,
    view_ids: &HashMap<u32, u16>,
    child_desktop_indices: &HashMap<u32, usize>,
    handle: u32,
) -> Option<Point> {
    let location = locate_live_view(view_ids, child_desktop_indices, handle)?;
    match location {
        LiveViewLocation::DesktopRoot(view_id) => {
            desktop.child_by_id(view_id).map(|view| view.bounds().a)
        }
        LiveViewLocation::DesktopChild {
            root_index,
            view_id,
        } => {
            let root = desktop.child_at_mut(root_index);
            if let Some(dialog) = root.as_any_mut().downcast_mut::<Dialog>() {
                return dialog.child_by_id(view_id).map(|view| view.bounds().a);
            }
            if let Some(window) = root.as_any_mut().downcast_mut::<Window>() {
                return window.child_by_id(view_id).map(|view| view.bounds().a);
            }
            None
        }
    }
}

fn patch_checked_handle(
    desktop: &mut Desktop,
    view_ids: &HashMap<u32, u16>,
    child_desktop_indices: &HashMap<u32, usize>,
    handle: u32,
) -> bool {
    let Some(location) = locate_live_view(view_ids, child_desktop_indices, handle) else {
        return false;
    };
    with_live_view_mut(desktop, location, |view| {
        if let Some(check_box) = view.as_any_mut().downcast_mut::<BridgedCheckBox>() {
            check_box.sync_from_cell();
            return true;
        }
        if let Some(radio) = view.as_any_mut().downcast_mut::<BridgedRadioButton>() {
            radio.sync_from_cell();
            return true;
        }
        false
    })
}

fn patch_list_items(
    desktop: &mut Desktop,
    view_ids: &HashMap<u32, u16>,
    child_desktop_indices: &HashMap<u32, usize>,
    handle: u32,
    items: Vec<String>,
    selection: Option<usize>,
) -> bool {
    let Some(location) = locate_live_view(view_ids, child_desktop_indices, handle) else {
        return false;
    };
    with_live_view_mut(desktop, location, |view| {
        if let Some(list_box) = view.as_any_mut().downcast_mut::<BridgedListBox>() {
            list_box.set_items_from_fpas(items, selection);
            return true;
        }
        if let Some(list_box) = view.as_any_mut().downcast_mut::<ListBox>() {
            list_box.set_items(items);
            if let Some(index) = selection {
                list_box.set_selection(index);
            }
            return true;
        }
        false
    })
}

impl Worker {
    fn turbo_vision_radio_group_handles(&self, handle: u32) -> Vec<u32> {
        self.with_tui(|tui| {
            let group_id = match tui.turbo_vision.objects.get(&handle) {
                Some(TurboVisionObject::RadioButton(radio)) => radio.group_id,
                _ => return Vec::new(),
            };
            tui.turbo_vision
                .objects
                .iter()
                .filter_map(|(member_handle, object)| {
                    let TurboVisionObject::RadioButton(radio) = object else {
                        return None;
                    };
                    (radio.group_id == group_id).then_some(*member_handle)
                })
                .collect()
        })
    }
}

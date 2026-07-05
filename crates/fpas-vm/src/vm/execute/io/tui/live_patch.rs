//! Incremental live turbo-vision view updates for FPAS data mutations.
//!
//! Avoids a full desktop rebuild when upstream setters can mirror FPAS handle state.
//!
//! **Documentation:** `docs/refactor/tui-bridge/05-reduce-reconcile-rebuild.md`

use super::bridged_check_box::BridgedCheckBox;
use super::bridged_list_box::BridgedListBox;
use super::bridged_memo::BridgedMemo;
use super::bridged_radio_button::BridgedRadioButton;
use super::bridged_static_text::BridgedStaticText;
use super::bridged_text_viewer::BridgedTextViewer;
use crate::vm::Worker;
use crate::vm::shared::TurboVisionObject;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
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
    SetText {
        handle: u32,
    },
}

enum LiveViewLocation {
    DesktopRoot(ViewId),
    DesktopChild {
        root_view_id: ViewId,
        view_id: ViewId,
    },
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
        let child_root_view_ids =
            self.with_tui(|tui| tui.turbo_vision.live_child_root_view_ids.clone());
        let mut app_slot = self.headless_tv_app.take();
        let Some(app) = app_slot.as_mut() else {
            self.headless_tv_app = app_slot;
            return false;
        };
        let ok = apply_live_data_mutation_to_desktop(
            app.desktop_mut(),
            &view_ids,
            &child_root_view_ids,
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
        let child_root_view_ids =
            self.with_tui(|tui| tui.turbo_vision.live_child_root_view_ids.clone());
        let mut app = match self.live_turbo_vision_app.take() {
            Some(app) => app,
            None => return false,
        };
        let ok = apply_live_data_mutation_to_desktop(
            &mut app.desktop,
            &view_ids,
            &child_root_view_ids,
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
        root_view_id: ViewId,
        view_id: ViewId,
    ) {
        self.with_tui(|tui| {
            tui.turbo_vision
                .live_view_ids
                .insert(handle, view_id.as_u16());
            tui.turbo_vision
                .live_child_root_view_ids
                .insert(handle, root_view_id.as_u16());
        });
    }

    pub(in crate::vm::execute::io::tui) fn turbo_vision_clear_live_view_ids(&self) {
        self.input_line_view_bindings.borrow_mut().clear();
        self.with_tui(|tui| {
            tui.turbo_vision.live_view_ids.clear();
            tui.turbo_vision.live_child_root_view_ids.clear();
        });
    }

    /// Remember the `InputLine` view binding built during desktop populate.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_register_input_line_view_binding(
        &self,
        handle: u32,
        binding: Rc<RefCell<String>>,
    ) {
        self.input_line_view_bindings
            .borrow_mut()
            .insert(handle, binding);
    }

    /// Mirror host `SetText` into the live turbo-vision `InputLine` buffer.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_sync_input_line_view_binding(
        &self,
        handle: u32,
        text: &str,
    ) {
        if let Some(binding) = self.input_line_view_bindings.borrow().get(&handle) {
            *binding.borrow_mut() = text.to_string();
        }
    }

    /// Top-left desktop coordinate for a registered live view (headless mouse routing).
    pub(in crate::vm::execute::io::tui) fn turbo_vision_live_view_click_point(
        &mut self,
        handle: u32,
    ) -> Option<Point> {
        let view_ids = self.with_tui(|tui| tui.turbo_vision.live_view_ids.clone());
        let child_root_view_ids =
            self.with_tui(|tui| tui.turbo_vision.live_child_root_view_ids.clone());
        let mut app_slot = self.headless_tv_app.take();
        let point = app_slot.as_mut().and_then(|app| {
            live_view_bounds_origin(app.desktop_mut(), &view_ids, &child_root_view_ids, handle)
        });
        self.headless_tv_app = app_slot;
        point
    }
}

fn apply_live_data_mutation_to_desktop(
    desktop: &mut turbo_vision::views::desktop::Desktop,
    view_ids: &HashMap<u32, u16>,
    child_root_view_ids: &HashMap<u32, u16>,
    mutation: &LiveDataMutation,
    worker: &Worker,
) -> bool {
    match mutation {
        LiveDataMutation::SetTitle { handle, title } => {
            let Some(location) = locate_live_view(view_ids, child_root_view_ids, *handle) else {
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
            targets
                .iter()
                .all(|&member| patch_checked_handle(desktop, view_ids, child_root_view_ids, member))
        }
        LiveDataMutation::SetListItems {
            handle,
            items,
            selection,
        } => patch_list_items(
            desktop,
            view_ids,
            child_root_view_ids,
            *handle,
            items.clone(),
            *selection,
        ),
        LiveDataMutation::SetText { handle } => {
            patch_set_text(desktop, view_ids, child_root_view_ids, *handle, worker)
        }
    }
}

fn view_id_for_handle(view_ids: &HashMap<u32, u16>, handle: u32) -> Option<ViewId> {
    view_ids.get(&handle).copied().map(ViewId::from_u16)
}

fn locate_live_view(
    view_ids: &HashMap<u32, u16>,
    child_root_view_ids: &HashMap<u32, u16>,
    handle: u32,
) -> Option<LiveViewLocation> {
    let view_id = view_id_for_handle(view_ids, handle)?;
    if let Some(root_view_id) = child_root_view_ids.get(&handle) {
        return Some(LiveViewLocation::DesktopChild {
            root_view_id: ViewId::from_u16(*root_view_id),
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
            root_view_id,
            view_id,
        } => {
            let Some(root) = desktop.child_by_id_mut(root_view_id) else {
                return false;
            };
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
    child_root_view_ids: &HashMap<u32, u16>,
    handle: u32,
) -> Option<Point> {
    let location = locate_live_view(view_ids, child_root_view_ids, handle)?;
    match location {
        LiveViewLocation::DesktopRoot(view_id) => {
            desktop.child_by_id(view_id).map(|view| view.bounds().a)
        }
        LiveViewLocation::DesktopChild {
            root_view_id,
            view_id,
        } => {
            let root = desktop.child_by_id_mut(root_view_id)?;
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
    child_root_view_ids: &HashMap<u32, u16>,
    handle: u32,
) -> bool {
    let Some(location) = locate_live_view(view_ids, child_root_view_ids, handle) else {
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
    child_root_view_ids: &HashMap<u32, u16>,
    handle: u32,
    items: Vec<String>,
    selection: Option<usize>,
) -> bool {
    let Some(location) = locate_live_view(view_ids, child_root_view_ids, handle) else {
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

fn patch_set_text(
    desktop: &mut Desktop,
    view_ids: &HashMap<u32, u16>,
    child_root_view_ids: &HashMap<u32, u16>,
    handle: u32,
    worker: &Worker,
) -> bool {
    if worker.with_tui(|tui| {
        matches!(
            tui.turbo_vision.objects.get(&handle),
            Some(TurboVisionObject::InputLine(_))
        )
    }) {
        return locate_live_view(view_ids, child_root_view_ids, handle).is_some();
    }

    let Some(text) = worker.turbo_vision_text_for_set_text(handle) else {
        return false;
    };
    let Some(location) = locate_live_view(view_ids, child_root_view_ids, handle) else {
        return false;
    };
    with_live_view_mut(desktop, location, |view| {
        if let Some(static_text) = view.as_any_mut().downcast_mut::<BridgedStaticText>() {
            static_text.set_text_from_fpas(&text);
            return true;
        }
        if let Some(memo) = view.as_any_mut().downcast_mut::<BridgedMemo>() {
            memo.set_text_from_fpas(&text);
            return true;
        }
        if let Some(text_viewer) = view.as_any_mut().downcast_mut::<BridgedTextViewer>() {
            text_viewer.set_text_from_fpas(&text);
            return true;
        }
        false
    })
}

impl Worker {
    fn turbo_vision_text_for_set_text(&self, handle: u32) -> Option<String> {
        self.with_tui(|tui| match tui.turbo_vision.objects.get(&handle) {
            Some(TurboVisionObject::StaticText(static_text)) => Some(static_text.text.clone()),
            Some(TurboVisionObject::Memo(memo)) => Some(memo.text.clone()),
            Some(TurboVisionObject::TextViewer(text_viewer)) => Some(text_viewer.text.clone()),
            _ => None,
        })
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use turbo_vision::core::geometry::Rect;

    fn sample_desktop_with_two_windows() -> (Desktop, ViewId, ViewId, ViewId, ViewId) {
        let mut desktop = Desktop::new(Rect::new(0, 0, 80, 25));
        let mut back = Window::new(Rect::new(5, 3, 30, 10), "Back");
        let back_child = back.add(Box::new(BridgedStaticText::new(
            Rect::new(2, 2, 10, 1),
            "BACK",
        )));
        let back_root = desktop.add(Box::new(back));

        let mut front = Window::new(Rect::new(10, 5, 30, 10), "Front");
        let front_child = front.add(Box::new(BridgedStaticText::new(
            Rect::new(2, 2, 10, 1),
            "FRONT",
        )));
        let front_root = desktop.add(Box::new(front));

        (desktop, back_root, back_child, front_root, front_child)
    }

    #[test]
    fn child_live_patch_survives_desktop_bring_to_front() {
        let (mut desktop, back_root, back_child, front_root, _front_child) =
            sample_desktop_with_two_windows();
        assert!(desktop.bring_to_front(front_root));

        let view_ids = HashMap::from([(1u32, back_child.as_u16())]);
        let root_view_ids = HashMap::from([(1u32, back_root.as_u16())]);

        let location = locate_live_view(&view_ids, &root_view_ids, 1).expect("locate child");
        assert!(with_live_view_mut(&mut desktop, location, |view| {
            view.as_any_mut()
                .downcast_mut::<BridgedStaticText>()
                .is_some()
        }));
    }

    #[test]
    fn locate_live_view_requires_correct_parent_root_view_id() {
        let (mut desktop, back_root, back_child, front_root, _front_child) =
            sample_desktop_with_two_windows();
        assert!(desktop.bring_to_front(back_root));

        let view_ids = HashMap::from([(42u32, back_child.as_u16())]);
        let mut root_view_ids = HashMap::from([(42u32, back_root.as_u16())]);

        assert!(with_live_view_mut(
            &mut desktop,
            locate_live_view(&view_ids, &root_view_ids, 42).unwrap(),
            |view| view
                .as_any_mut()
                .downcast_mut::<BridgedStaticText>()
                .is_some(),
        ));

        root_view_ids.insert(42, front_root.as_u16());
        assert!(!with_live_view_mut(
            &mut desktop,
            locate_live_view(&view_ids, &root_view_ids, 42).unwrap(),
            |view| view
                .as_any_mut()
                .downcast_mut::<BridgedStaticText>()
                .is_some(),
        ));
    }
}

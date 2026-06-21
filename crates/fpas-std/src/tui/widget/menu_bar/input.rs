//! Menu bar mouse and keyboard routing.
//!
//! Spec: `docs/pascal/std/tui/app/README.md`

use crate::key_event::{ConsoleKeyEvent, key_kind_index};
use crate::mouse_action_index;
use crate::{CommandId, UiMouse, ViewRect};

use super::super::menu_popup::{
    popup_alt_shortcut_index, popup_entry_at, popup_entry_is_actionable, popup_entry_is_selectable,
    popup_shortcut_index,
};
use super::MenuBarWidget;
use super::geometry::{has_submenu, item_index_at, view_mouse_coords};
use super::types::{MenuBarMouseResult, OpenSubmenu};

impl MenuBarWidget {
    /// Route a mouse event within the menu bar and open popup regions.
    pub fn handle_mouse(&mut self, rect: ViewRect, mouse: UiMouse) -> MenuBarMouseResult {
        let (mouse_x, mouse_y) = view_mouse_coords(mouse);
        let down = mouse.action == mouse_action_index("Down");

        if let Some(open) = self.open_submenu {
            let Some(popup) = self.open_popup_rect(rect) else {
                return MenuBarMouseResult::Ignored;
            };
            if popup.contains_view_point(mouse_x, mouse_y) {
                let entries = &self.items[open.bar_index].submenu;
                let entry_index = popup_entry_at(popup, entries, mouse_x, mouse_y);
                if !down {
                    if mouse.action == mouse_action_index("Move") {
                        return self.hover_submenu_entry(open.bar_index, entry_index);
                    }
                    return MenuBarMouseResult::Ignored;
                }
                let Some(entry_index) = entry_index else {
                    return MenuBarMouseResult::Ignored;
                };
                let (command_id, enabled, separator) = {
                    let entry = &entries[entry_index];
                    (entry.command_id, entry.enabled, entry.separator)
                };
                if separator || !enabled || command_id < 0 {
                    return MenuBarMouseResult::Ignored;
                }
                self.close_submenu();
                return MenuBarMouseResult::Command(CommandId(command_id));
            }
            if down {
                self.close_submenu();
                return MenuBarMouseResult::HoverChanged;
            }
        }

        if rect.contains_console_mouse(mouse.x, mouse.y) {
            let item_index = item_index_at(self.items.as_slice(), rect, mouse_x);
            if down {
                if let Some(index) = item_index {
                    let item = &self.items[index];
                    if !item.enabled {
                        return MenuBarMouseResult::Ignored;
                    }
                    if has_submenu(item) {
                        return self.toggle_submenu(index);
                    }
                    if item.command_id >= 0 {
                        self.hovered = Some(index);
                        self.menu_active = true;
                        return MenuBarMouseResult::Command(CommandId(item.command_id));
                    }
                }
                return MenuBarMouseResult::Ignored;
            }

            if self.hovered != item_index {
                self.hovered = item_index.filter(|index| self.items[*index].enabled);
                return MenuBarMouseResult::HoverChanged;
            }
            return MenuBarMouseResult::Ignored;
        }

        if self.hovered.take().is_some() || self.open_submenu.take().is_some() {
            self.menu_active = false;
            return MenuBarMouseResult::HoverChanged;
        }
        MenuBarMouseResult::Ignored
    }

    /// Route Alt+letter shortcuts, F10 menu activation, and popup navigation keys.
    pub fn handle_key(&mut self, key: &ConsoleKeyEvent) -> MenuBarMouseResult {
        if let Some(result) = self.handle_submenu_key(key) {
            return result;
        }
        if self.handle_menu_navigation_key(key) {
            return MenuBarMouseResult::HoverChanged;
        }

        if key.kind == key_kind_index("F10") && !key.ctrl && !key.alt && !key.meta {
            return self.activate_menu_mode();
        }

        let Some(shortcut) = shortcut_letter(key) else {
            return MenuBarMouseResult::Ignored;
        };

        let Some(index) = self
            .items
            .iter()
            .position(|item| item.enabled && item_matches_shortcut(item, shortcut))
        else {
            return MenuBarMouseResult::Ignored;
        };

        self.menu_active = true;
        self.hovered = Some(index);
        let item = &self.items[index];
        if has_submenu(item) {
            return self.open_submenu_at(index);
        }
        if item.command_id >= 0 {
            return MenuBarMouseResult::Command(CommandId(item.command_id));
        }
        MenuBarMouseResult::HoverChanged
    }

    fn handle_submenu_key(&mut self, key: &ConsoleKeyEvent) -> Option<MenuBarMouseResult> {
        let open = self.open_submenu?;

        if key.kind == key_kind_index("Escape") && !key.ctrl && !key.alt && !key.meta {
            self.close_submenu();
            return Some(MenuBarMouseResult::HoverChanged);
        }

        if key.kind == key_kind_index("Up") && !key.ctrl && !key.alt && !key.meta {
            self.move_popup_selection(-1);
            return Some(MenuBarMouseResult::HoverChanged);
        }
        if key.kind == key_kind_index("Down") && !key.ctrl && !key.alt && !key.meta {
            self.move_popup_selection(1);
            return Some(MenuBarMouseResult::HoverChanged);
        }
        if key.kind == key_kind_index("Enter") && !key.ctrl && !key.alt && !key.meta {
            let entry = &self.items[open.bar_index].submenu[open.entry_index];
            if popup_entry_is_actionable(entry) {
                let command_id = entry.command_id;
                self.close_submenu();
                return Some(MenuBarMouseResult::Command(CommandId(command_id)));
            }
            return Some(MenuBarMouseResult::Ignored);
        }

        let entries = &self.items[open.bar_index].submenu;
        if let Some(index) = popup_alt_shortcut_index(entries, key)
            .or_else(|| popup_shortcut_key_index(entries, key))
        {
            let entry = &entries[index];
            if popup_entry_is_actionable(entry) {
                let command_id = entry.command_id;
                self.close_submenu();
                return Some(MenuBarMouseResult::Command(CommandId(command_id)));
            }
        }

        None
    }

    fn handle_menu_navigation_key(&mut self, key: &ConsoleKeyEvent) -> bool {
        if !self.menu_active || self.open_submenu.is_some() {
            return false;
        }
        if key.ctrl || key.alt || key.meta {
            return false;
        }

        match key.kind {
            k if k == key_kind_index("Escape") => {
                self.menu_active = false;
                self.hovered = None;
                true
            }
            k if k == key_kind_index("Left") => self.move_bar_selection(-1),
            k if k == key_kind_index("Right") => self.move_bar_selection(1),
            k if k == key_kind_index("Down") => self
                .hovered
                .and_then(|index| {
                    if has_submenu(&self.items[index]) {
                        self.open_submenu_at(index);
                        Some(true)
                    } else {
                        None
                    }
                })
                .unwrap_or(false),
            _ => false,
        }
    }

    fn activate_menu_mode(&mut self) -> MenuBarMouseResult {
        self.menu_active = true;
        let index = self.items.iter().position(|item| item.enabled);
        let Some(index) = index else {
            return MenuBarMouseResult::Ignored;
        };
        self.hovered = Some(index);
        if has_submenu(&self.items[index]) {
            return self.open_submenu_at(index);
        }
        MenuBarMouseResult::HoverChanged
    }

    fn toggle_submenu(&mut self, index: usize) -> MenuBarMouseResult {
        if self
            .open_submenu
            .is_some_and(|open| open.bar_index == index)
        {
            self.close_submenu();
            MenuBarMouseResult::HoverChanged
        } else {
            self.open_submenu_at(index)
        }
    }

    fn open_submenu_at(&mut self, index: usize) -> MenuBarMouseResult {
        let first_selectable = self.items[index]
            .submenu
            .iter()
            .position(popup_entry_is_selectable)
            .unwrap_or(0);
        self.hovered = Some(index);
        self.menu_active = true;
        self.open_submenu = Some(OpenSubmenu {
            bar_index: index,
            entry_index: first_selectable,
        });
        MenuBarMouseResult::HoverChanged
    }

    fn close_submenu(&mut self) {
        self.open_submenu = None;
    }

    fn move_bar_selection(&mut self, delta: i64) -> bool {
        let Some(current) = self.hovered else {
            return false;
        };
        let len = self.items.len();
        if len == 0 {
            return false;
        }
        let mut next = current as i64;
        for _ in 0..len {
            next = (next + delta).rem_euclid(len as i64);
            let index = next as usize;
            if self.items[index].enabled {
                if self.hovered == Some(index) {
                    return false;
                }
                self.hovered = Some(index);
                self.sync_submenu_for_bar_index(index);
                return true;
            }
        }
        false
    }

    fn sync_submenu_for_bar_index(&mut self, index: usize) {
        if !self.menu_active {
            return;
        }
        if has_submenu(&self.items[index]) {
            let entry_index = self.items[index]
                .submenu
                .iter()
                .position(popup_entry_is_selectable)
                .unwrap_or(0);
            self.open_submenu = Some(OpenSubmenu {
                bar_index: index,
                entry_index,
            });
        } else {
            self.close_submenu();
        }
    }

    fn move_popup_selection(&mut self, delta: i64) {
        let Some(open) = self.open_submenu.as_mut() else {
            return;
        };
        let entries = &self.items[open.bar_index].submenu;
        let len = entries.len();
        if len == 0 {
            return;
        }
        let mut next = open.entry_index as i64;
        for _ in 0..len {
            next = (next + delta).rem_euclid(len as i64);
            let index = next as usize;
            if popup_entry_is_selectable(&entries[index]) {
                open.entry_index = index;
                return;
            }
        }
    }

    /// Open the pull-down for the currently hovered top-level item, when one exists.
    pub fn open_hovered_submenu(&mut self) -> MenuBarMouseResult {
        let Some(index) = self.hovered else {
            return MenuBarMouseResult::Ignored;
        };
        let Some(item) = self.items.get(index).filter(|item| item.enabled) else {
            return MenuBarMouseResult::Ignored;
        };
        if !has_submenu(item) {
            return MenuBarMouseResult::Ignored;
        }
        if self
            .open_submenu
            .is_some_and(|open| open.bar_index == index)
        {
            return MenuBarMouseResult::Ignored;
        }
        self.open_submenu_at(index)
    }

    /// Clear hover, open submenu, and menu-mode flags after terminal focus loss.
    pub fn clear_transient_pointer_state(&mut self) -> bool {
        let changed = self.hovered.is_some() || self.open_submenu.is_some() || self.menu_active;
        self.hovered = None;
        self.open_submenu = None;
        self.menu_active = false;
        changed
    }

    /// Clear hover and any open pull-down when the pointer leaves the bar and popups.
    pub fn clear_pointer_hover_outside(&mut self, bar_rect: ViewRect, mouse: UiMouse) -> bool {
        if self.contains_point(bar_rect, mouse.x, mouse.y) {
            return false;
        }
        let changed = self.hovered.is_some() || self.open_submenu.is_some();
        self.hovered = None;
        self.open_submenu = None;
        if changed {
            self.menu_active = false;
        }
        changed
    }

    fn hover_submenu_entry(
        &mut self,
        bar_index: usize,
        entry_index: Option<usize>,
    ) -> MenuBarMouseResult {
        let Some(entry_index) = entry_index else {
            return MenuBarMouseResult::Ignored;
        };
        let entry = &self.items[bar_index].submenu[entry_index];
        if !popup_entry_is_selectable(entry) {
            return MenuBarMouseResult::Ignored;
        }
        if self
            .open_submenu
            .is_some_and(|open| open.bar_index == bar_index && open.entry_index == entry_index)
        {
            return MenuBarMouseResult::Ignored;
        }
        if let Some(open) = self.open_submenu.as_mut() {
            open.entry_index = entry_index;
        }
        MenuBarMouseResult::HoverChanged
    }
}

fn popup_shortcut_key_index(
    entries: &[super::super::menu_popup::MenuPopupItem],
    key: &ConsoleKeyEvent,
) -> Option<usize> {
    if key.ctrl || key.alt || key.meta || key.kind != key_kind_index("Character") {
        return None;
    }
    popup_shortcut_index(entries, key.ch)
}

fn shortcut_letter(key: &ConsoleKeyEvent) -> Option<char> {
    if !key.alt || key.ctrl || key.meta || key.kind != key_kind_index("Character") {
        return None;
    }
    key.ch
        .is_ascii_alphabetic()
        .then_some(key.ch.to_ascii_lowercase())
}

fn item_matches_shortcut(item: &super::types::MenuBarItem, shortcut: char) -> bool {
    item.shortcut
        .chars()
        .next()
        .is_some_and(|letter| letter.eq_ignore_ascii_case(&shortcut))
}

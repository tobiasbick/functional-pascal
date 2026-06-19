//! Menu bar layout, hit-testing, and damage-region geometry.
//!
//! Spec: `docs/pascal/std/tui/app/README.md`

use crate::{DamageRegion, UiMouse, ViewRect};

use super::super::menu_popup::{MenuPopupItem, MenuPopupRect, popup_rect};
use super::MenuBarWidget;
use super::types::MenuBarItem;

impl MenuBarWidget {
    /// Returns terminal rectangles that may need redraw for the current state.
    #[must_use]
    pub fn damage_rects(&self, bar_rect: ViewRect) -> Vec<ViewRect> {
        let mut rects = vec![bar_rect];
        if let Some(popup) = self.open_popup_rect(bar_rect) {
            rects.push(popup.as_view_rect());
        }
        rects
    }

    /// Returns whether a point hits the bar row or an open pull-down menu.
    ///
    /// `mouse_x`/`mouse_y` use one-based console coordinates.
    #[must_use]
    pub fn contains_point(&self, bar_rect: ViewRect, mouse_x: i64, mouse_y: i64) -> bool {
        bar_rect.contains_console_mouse(mouse_x, mouse_y)
            || self.open_popup_rect(bar_rect).is_some_and(|popup| {
                popup.contains_view_point(view_mouse_x(mouse_x), view_mouse_y(mouse_y))
            })
    }

    pub(super) fn is_bar_item_active(&self, index: usize) -> bool {
        self.hovered == Some(index)
            || self
                .open_submenu
                .is_some_and(|open| open.bar_index == index)
    }

    pub(super) fn open_popup(
        &self,
        bar_rect: ViewRect,
    ) -> Option<(MenuPopupRect, &[MenuPopupItem], usize)> {
        let open = self.open_submenu?;
        let entries = &self.items[open.bar_index].submenu;
        if entries.is_empty() {
            return None;
        }
        let anchor_x = item_x_at(self.items.as_slice(), bar_rect, open.bar_index)?;
        let popup = popup_rect(anchor_x, bar_rect.y + bar_rect.height, entries);
        Some((popup, entries.as_slice(), open.entry_index))
    }

    pub(super) fn open_popup_rect(&self, bar_rect: ViewRect) -> Option<MenuPopupRect> {
        self.open_popup(bar_rect).map(|(popup, _, _)| popup)
    }
}

pub(super) fn view_mouse_x(mouse_x: i64) -> i64 {
    mouse_x.saturating_sub(1)
}

pub(super) fn view_mouse_y(mouse_y: i64) -> i64 {
    mouse_y.saturating_sub(1)
}

pub(super) fn view_mouse_coords(mouse: UiMouse) -> (i64, i64) {
    (view_mouse_x(mouse.x), view_mouse_y(mouse.y))
}

pub(super) fn has_submenu(item: &MenuBarItem) -> bool {
    !item.submenu.is_empty()
}

pub(super) fn item_index_at(items: &[MenuBarItem], rect: ViewRect, mouse_x: i64) -> Option<usize> {
    let mut x = rect.x;
    for (index, item) in items.iter().enumerate() {
        let width = item_display_width(item);
        if width <= 0 {
            continue;
        }
        let end = x.saturating_add(width);
        if mouse_x >= x && mouse_x < end {
            return Some(index);
        }
        x = end;
    }
    None
}

pub(super) fn item_x_at(items: &[MenuBarItem], rect: ViewRect, index: usize) -> Option<i64> {
    let mut x = rect.x;
    for (current, item) in items.iter().enumerate() {
        if current == index {
            return Some(x);
        }
        x = x.saturating_add(item_display_width(item));
    }
    None
}

pub(super) fn item_display_width(item: &MenuBarItem) -> i64 {
    (item.label.chars().count() as i64).saturating_add(2)
}

pub(super) fn intersects_damage(rect: ViewRect, damage: DamageRegion) -> bool {
    match damage {
        DamageRegion::FullFrame => true,
        DamageRegion::Rect(dirty) => rect.intersects(dirty),
    }
}

pub(super) fn clip_rect_to_damage(rect: ViewRect, damage: DamageRegion) -> Option<ViewRect> {
    match damage {
        DamageRegion::FullFrame => Some(rect),
        DamageRegion::Rect(dirty) => rect.intersection(dirty),
    }
}

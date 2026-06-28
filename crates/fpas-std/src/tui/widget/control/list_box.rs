//! Retained scrolling list-box widget.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use super::{paint_chars, truncated_chars};
use crate::{CommandId, Console, DamageRegion, ScrollModel, ViewRect};

const EMPTY_PLACEHOLDER: &str = "(empty)";

/// One selectable list-box row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListBoxItem {
    /// Row text.
    pub text: String,
    /// Command dispatched when the row is activated.
    pub command_id: Option<CommandId>,
    /// Whether the row can be selected or activated.
    pub enabled: bool,
}
/// CRT colors for list boxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListBoxStyle {
    /// Background color.
    pub bg: u8,
    /// Normal foreground color.
    pub fg: u8,
    /// Selected-row background color.
    pub active_bg: u8,
    /// Selected-row foreground color.
    pub active_fg: u8,
    /// Disabled-row foreground color.
    pub disabled_fg: u8,
}
impl Default for ListBoxStyle {
    fn default() -> Self {
        Self {
            bg: 7,
            fg: 0,
            active_bg: 0,
            active_fg: 15,
            disabled_fg: 8,
        }
    }
}
/// Vertically scrolling retained list control.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListBoxWidget {
    /// Current rows.
    pub items: Vec<ListBoxItem>,
    selected: Option<usize>,
    /// Whether the control accepts interaction.
    pub enabled: bool,
    /// Whether the control owns keyboard focus.
    pub focused: bool,
    /// Current paint style.
    pub style: ListBoxStyle,
    scroll: ScrollModel,
}
impl ListBoxWidget {
    /// Create a list selecting its first enabled row.
    #[must_use]
    pub fn new(items: Vec<ListBoxItem>, viewport: usize) -> Self {
        let selected = items.iter().position(|i| i.enabled);
        Self {
            scroll: ScrollModel::new(items.len(), viewport),
            items,
            selected,
            enabled: true,
            focused: false,
            style: ListBoxStyle::default(),
        }
    }
    /// Return selected row.
    #[must_use]
    pub const fn selected(&self) -> Option<usize> {
        self.selected
    }
    /// Return first visible row.
    #[must_use]
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }
    /// Return selected command.
    #[must_use]
    pub fn selected_command(&self) -> Option<CommandId> {
        self.selected
            .and_then(|i| self.items.get(i))
            .and_then(|i| i.command_id)
    }
    /// Replace rows and reset selection.
    pub fn set_items(&mut self, items: Vec<ListBoxItem>, viewport: usize) {
        self.items = items;
        self.selected = self.items.iter().position(|i| i.enabled);
        self.scroll = ScrollModel::new(self.items.len(), viewport);
    }
    /// Select an enabled row and reveal it.
    pub fn set_selected(&mut self, index: usize) -> bool {
        if !self.enabled || !self.items.get(index).is_some_and(|i| i.enabled) {
            return false;
        }
        let changed = self.selected != Some(index);
        self.selected = Some(index);
        self.scroll.ensure_visible(index);
        changed
    }
    /// Move selection to the next enabled row in one direction.
    pub fn move_selection(&mut self, forward: bool) -> bool {
        let Some(current) = self.selected else {
            return false;
        };
        let mut i = current as i64 + if forward { 1 } else { -1 };
        while i >= 0 && (i as usize) < self.items.len() {
            if self.items[i as usize].enabled {
                return self.set_selected(i as usize);
            }
            i += if forward { 1 } else { -1 };
        }
        false
    }
    /// Select the first or last enabled row.
    pub fn select_edge(&mut self, last: bool) -> bool {
        let found = if last {
            self.items.iter().rposition(|i| i.enabled)
        } else {
            self.items.iter().position(|i| i.enabled)
        };
        found.is_some_and(|i| self.set_selected(i))
    }
    /// Scroll without changing selection.
    pub fn scroll_by(&mut self, delta: i64) -> bool {
        self.scroll.scroll_by(delta)
    }
    /// Select a visible zero-based row.
    pub fn select_visible_row(&mut self, row: usize) -> bool {
        self.set_selected(self.scroll.offset().saturating_add(row))
    }
    /// Paint visible rows.
    pub fn paint(&self, c: &mut Console, r: ViewRect, d: DamageRegion) {
        let Some(clip) = d.clip_rect(r) else {
            return;
        };
        c.fill_rect_crt(clip, self.style.fg, self.style.bg, ' ');
        if self.items.is_empty() && r.height > 0 {
            let rr = ViewRect {
                x: r.x,
                y: r.y,
                width: r.width,
                height: 1,
            };
            paint_chars(
                c,
                rr,
                clip,
                truncated_chars(EMPTY_PLACEHOLDER, rr.width),
                |_| self.style.disabled_fg,
                self.style.bg,
            );
            return;
        }
        for (row, item) in self
            .items
            .iter()
            .skip(self.scroll.offset())
            .take(r.height.max(0) as usize)
            .enumerate()
        {
            let index = self.scroll.offset() + row;
            let active = self.focused && self.selected == Some(index) && item.enabled;
            let (fg, bg) = if !item.enabled {
                (self.style.disabled_fg, self.style.bg)
            } else if active {
                (self.style.active_fg, self.style.active_bg)
            } else {
                (self.style.fg, self.style.bg)
            };
            let rr = ViewRect {
                x: r.x,
                y: r.y + row as i64,
                width: r.width,
                height: 1,
            };
            paint_chars(
                c,
                rr,
                clip,
                truncated_chars(&item.text, rr.width),
                |_| fg,
                bg,
            );
        }
    }
}

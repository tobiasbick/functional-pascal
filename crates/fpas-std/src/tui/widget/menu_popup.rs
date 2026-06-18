//! Pull-down menu entries painted below a menu bar item.
//!
//! Spec: `docs/pascal/std/tui/app/README.md`

use crate::key_event::{ConsoleKeyEvent, key_kind_index};
use crate::{Console, ViewRect};

use super::menu_label_paint::paint_labeled_text;
use super::menu_style::{MenuBarStyle, MenuLabelPaint};

/// One pull-down entry supplied from Pascal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuPopupItem {
    /// Visible menu text.
    pub label: String,
    /// Letter shortcut while the popup is open (case-insensitive). Empty means none.
    pub shortcut: String,
    /// When false, the entry is drawn disabled and ignores activation.
    pub enabled: bool,
    /// Command dispatched through `OnCommand` on activation.
    pub command_id: i64,
    /// When true, draws a horizontal rule and ignores keyboard and mouse activation.
    pub separator: bool,
}

/// Returns whether a popup row can be highlighted or activated.
#[must_use]
pub fn popup_entry_is_selectable(entry: &MenuPopupItem) -> bool {
    !entry.separator
}

/// Returns whether a popup row dispatches `OnCommand`.
#[must_use]
pub fn popup_entry_is_actionable(entry: &MenuPopupItem) -> bool {
    popup_entry_is_selectable(entry) && entry.enabled && entry.command_id >= 0
}

/// Absolute terminal rectangle for an open pull-down menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuPopupRect {
    /// Left edge in zero-based terminal coordinates.
    pub x: i64,
    /// Top edge in zero-based terminal coordinates.
    pub y: i64,
    /// Inner width excluding the border columns.
    pub inner_width: i64,
    /// Number of visible entry rows.
    pub height: i64,
}

impl MenuPopupRect {
    /// Total width including left and right border columns.
    #[must_use]
    pub fn outer_width(&self) -> i64 {
        self.inner_width.saturating_add(2)
    }

    /// Total height including top and bottom border rows.
    #[must_use]
    pub fn outer_height(&self) -> i64 {
        self.height.saturating_add(2)
    }

    /// Converts to a [`ViewRect`] covering the full framed popup.
    #[must_use]
    pub fn as_view_rect(self) -> ViewRect {
        ViewRect {
            x: self.x,
            y: self.y,
            width: self.outer_width(),
            height: self.outer_height(),
        }
    }

    /// Returns whether `x`/`y` lie inside the framed popup.
    ///
    /// Coordinates are zero-based view space.
    #[must_use]
    pub fn contains_view_point(self, x: i64, y: i64) -> bool {
        self.as_view_rect().contains_point(x, y)
    }
}

/// Computes popup geometry for `entries` anchored under `anchor_x`.
#[must_use]
pub fn popup_rect(anchor_x: i64, bar_bottom_y: i64, entries: &[MenuPopupItem]) -> MenuPopupRect {
    let inner_width = entries
        .iter()
        .map(entry_display_width)
        .max()
        .unwrap_or(8)
        .max(8);
    MenuPopupRect {
        x: anchor_x,
        y: bar_bottom_y,
        inner_width,
        height: entries.len() as i64,
    }
}

/// Paints a framed pull-down menu.
pub fn paint_popup(
    console: &mut Console,
    popup: MenuPopupRect,
    entries: &[MenuPopupItem],
    style: MenuBarStyle,
    selected: usize,
) {
    let outer = popup.as_view_rect();
    paint_popup_frame(console, outer, style);

    for (index, entry) in entries.iter().enumerate() {
        let row_y = popup.y + 1 + index as i64;
        if entry.separator {
            paint_popup_separator(console, popup.x + 1, row_y, popup.inner_width, style);
            continue;
        }
        let label = pad_to_width(&format!(" {} ", entry.label), popup.inner_width);
        let selected_row = index == selected;
        let (fg, bg) = if !entry.enabled {
            (style.disabled_fg, style.bar_bg)
        } else if selected_row {
            (style.highlight_fg, style.highlight_bg)
        } else {
            (style.bar_fg, style.bar_bg)
        };
        paint_popup_label(
            console,
            popup.x + 1,
            row_y,
            entry,
            MenuLabelPaint {
                fg,
                bg,
                accel_fg: style.accel_fg,
                hovered: selected_row,
            },
            &label,
        );
    }
}

fn paint_popup_separator(
    console: &mut Console,
    x: i64,
    y: i64,
    inner_width: i64,
    style: MenuBarStyle,
) {
    for offset in 0..inner_width {
        console.write_char_at_crt(x + offset, y, '─', style.bar_fg, style.bar_bg);
    }
}

fn paint_popup_label(
    console: &mut Console,
    x: i64,
    y: i64,
    item: &MenuPopupItem,
    colors: MenuLabelPaint,
    label: &str,
) {
    paint_labeled_text(console, x, y, label, &item.shortcut, item.enabled, colors);
}

fn paint_popup_frame(console: &mut Console, outer: ViewRect, style: MenuBarStyle) {
    let fg = style.bar_fg;
    let bg = style.bar_bg;
    console.fill_rect_crt(outer, fg, bg, ' ');

    let right = outer.x + outer.width - 1;
    let bottom = outer.y + outer.height - 1;
    console.write_char_at_crt(outer.x, outer.y, '┌', fg, bg);
    console.write_char_at_crt(right, outer.y, '┐', fg, bg);
    console.write_char_at_crt(outer.x, bottom, '└', fg, bg);
    console.write_char_at_crt(right, bottom, '┘', fg, bg);

    for x in (outer.x + 1)..right {
        console.write_char_at_crt(x, outer.y, '─', fg, bg);
        console.write_char_at_crt(x, bottom, '─', fg, bg);
    }
    for y in (outer.y + 1)..bottom {
        console.write_char_at_crt(outer.x, y, '│', fg, bg);
        console.write_char_at_crt(right, y, '│', fg, bg);
    }
}

/// Returns the entry index under `mouse_x`/`mouse_y`, if any.
///
/// Coordinates are zero-based view space, matching [`MenuPopupRect::contains_view_point`].
#[must_use]
pub fn popup_entry_at(
    popup: MenuPopupRect,
    entries: &[MenuPopupItem],
    mouse_x: i64,
    mouse_y: i64,
) -> Option<usize> {
    if !popup.contains_view_point(mouse_x, mouse_y) {
        return None;
    }
    let row = mouse_y - popup.y - 1;
    if row < 0 || row >= entries.len() as i64 {
        return None;
    }
    Some(row as usize)
}

/// Returns the first enabled entry index matching an unmodified letter shortcut.
#[must_use]
pub fn popup_shortcut_index(entries: &[MenuPopupItem], ch: char) -> Option<usize> {
    if !ch.is_ascii_alphabetic() {
        return None;
    }
    entries.iter().position(|entry| {
        popup_entry_is_actionable(entry)
            && entry
                .shortcut
                .chars()
                .next()
                .is_some_and(|letter| letter.eq_ignore_ascii_case(&ch))
    })
}

/// Returns the first enabled entry index matching Alt+letter.
#[must_use]
pub fn popup_alt_shortcut_index(entries: &[MenuPopupItem], key: &ConsoleKeyEvent) -> Option<usize> {
    if !key.alt || key.ctrl || key.meta || key.kind != key_kind_index("Character") {
        return None;
    }
    popup_shortcut_index(entries, key.ch)
}

fn entry_display_width(entry: &MenuPopupItem) -> i64 {
    (entry.label.chars().count() as i64).saturating_add(2)
}

fn pad_to_width(text: &str, width: i64) -> String {
    let current = text.chars().count() as i64;
    if current >= width {
        return text.to_string();
    }
    format!("{text}{}", " ".repeat((width - current) as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_rect_sizes_to_longest_entry() {
        let entries = vec![
            MenuPopupItem {
                label: "Open".into(),
                shortcut: String::new(),
                enabled: true,
                command_id: 1,
                separator: false,
            },
            MenuPopupItem {
                label: "Exit".into(),
                shortcut: "X".into(),
                enabled: true,
                command_id: 2,
                separator: false,
            },
        ];
        let popup = popup_rect(3, 1, &entries);
        assert_eq!(popup.inner_width, 8);
        assert_eq!(popup.outer_height(), 4);
    }

    #[test]
    fn popup_entry_at_uses_view_coordinates() {
        let entries = vec![MenuPopupItem {
            label: "Exit".into(),
            shortcut: "X".into(),
            enabled: true,
            command_id: 2,
            separator: false,
        }];
        let popup = popup_rect(0, 1, &entries);
        assert_eq!(popup_entry_at(popup, &entries, 2, 2), Some(0));
        assert_eq!(popup_entry_at(popup, &entries, 2, 1), None);
    }

    #[test]
    fn popup_navigation_skips_separator_rows() {
        let entries = vec![
            MenuPopupItem {
                label: "Open".into(),
                shortcut: String::new(),
                enabled: false,
                command_id: -1,
                separator: false,
            },
            MenuPopupItem {
                label: String::new(),
                shortcut: String::new(),
                enabled: false,
                command_id: -1,
                separator: true,
            },
            MenuPopupItem {
                label: "Exit".into(),
                shortcut: "X".into(),
                enabled: true,
                command_id: 2,
                separator: false,
            },
        ];
        assert!(!popup_entry_is_selectable(&entries[1]));
        assert!(popup_entry_is_actionable(&entries[2]));
    }
}

//! Menu bar and label painting.
//!
//! Spec: `docs/pascal/std/tui-app.md`

use crate::{Console, DamageRegion, ViewRect};

use super::super::menu_popup::paint_popup;
use super::MenuBarWidget;
use super::geometry::{clip_rect_to_damage, intersects_damage};
use super::types::{MenuBarItem, MenuBarStyle, MenuLabelPaint};

impl MenuBarWidget {
    /// Paint the menu bar and any open pull-down clipped to `damage`.
    pub fn paint(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        if intersects_damage(rect, damage) {
            let Some(clip) = clip_rect_to_damage(rect, damage) else {
                return;
            };
            console.fill_rect_crt(clip, self.style.bar_fg, self.style.bar_bg, ' ');

            let mut x = rect.x;
            for (index, item) in self.items.iter().enumerate() {
                if x >= rect.x + rect.width {
                    break;
                }
                let label = format!(" {} ", item.label);
                let width = label.chars().count() as i64;
                if width <= 0 || x + width > rect.x + rect.width {
                    break;
                }

                let hovered = self.is_bar_item_active(index);
                let (fg, bg) = bar_item_colors(item, self.style, hovered);
                paint_bar_label(
                    console,
                    x,
                    rect.y,
                    item,
                    MenuLabelPaint {
                        fg,
                        bg,
                        accel_fg: self.style.accel_fg,
                        hovered,
                    },
                );
                x += width;
            }
        }
    }

    /// Paint an open pull-down menu above other views (second paint pass).
    pub fn paint_popup_overlay(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        if let Some((popup, entries, selected)) = self.open_popup(rect) {
            let popup_rect = popup.as_view_rect();
            if intersects_damage(popup_rect, damage) {
                paint_popup(console, popup, entries, self.style, selected);
            }
        }
    }
}

fn bar_item_colors(item: &MenuBarItem, style: MenuBarStyle, hovered: bool) -> (u8, u8) {
    if !item.enabled {
        (style.disabled_fg, style.bar_bg)
    } else if hovered {
        (style.highlight_fg, style.highlight_bg)
    } else {
        (style.bar_fg, style.bar_bg)
    }
}

/// Paint one top-level menu bar label with optional shortcut highlighting.
pub(in crate::tui::widget) fn paint_bar_label(
    console: &mut Console,
    x: i64,
    y: i64,
    item: &MenuBarItem,
    colors: MenuLabelPaint,
) {
    let label = format!(" {} ", item.label);
    paint_labeled_text(console, x, y, &label, &item.shortcut, item.enabled, colors);
}

/// Paint label text with Turbo Pascal-style shortcut letter highlighting.
pub(in crate::tui::widget) fn paint_labeled_text(
    console: &mut Console,
    x: i64,
    y: i64,
    label: &str,
    shortcut: &str,
    enabled: bool,
    colors: MenuLabelPaint,
) {
    let highlight_index = shortcut_highlight_index(label.trim(), shortcut);
    let mut col = x;
    for (index, ch) in label.chars().enumerate() {
        let cell_fg = if colors.hovered || !enabled {
            colors.fg
        } else if highlight_index == Some(index) {
            colors.accel_fg
        } else {
            colors.fg
        };
        console.write_char_at_crt(col, y, ch, cell_fg, colors.bg);
        col += 1;
    }
}

/// Returns the character index of the shortcut letter inside a padded label.
pub(in crate::tui::widget) fn shortcut_highlight_index(
    label: &str,
    shortcut: &str,
) -> Option<usize> {
    let shortcut = shortcut.chars().next()?;
    if !shortcut.is_ascii_alphabetic() {
        return None;
    }
    let inner = format!(" {label} ");
    inner
        .char_indices()
        .find(|(_, ch)| ch.eq_ignore_ascii_case(&shortcut))
        .map(|(index, _)| index)
}

//! Menu bar and label painting.
//!
//! Spec: `docs/pascal/std/tui/app/README.md`

use crate::text::str_display_width;
use crate::{Console, DamageRegion, ViewRect};

use super::super::menu_label_paint::paint_labeled_text;
use super::super::menu_popup::paint_popup;
use super::super::menu_style::{MenuBarStyle, MenuLabelPaint};
use super::MenuBarWidget;
use super::types::MenuBarItem;

impl MenuBarWidget {
    /// Paint the menu bar and any open pull-down clipped to `damage`.
    pub fn paint(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        if damage.intersects_rect(rect) {
            let Some(clip) = damage.clip_rect(rect) else {
                return;
            };
            console.fill_rect_crt(clip, self.style.bar_fg, self.style.bar_bg, ' ');

            let mut x = rect.x;
            for (index, item) in self.items.iter().enumerate() {
                if x >= rect.x + rect.width {
                    break;
                }
                let label = format!(" {} ", item.label);
                let width = str_display_width(&label);
                if width <= 0 || x + width > rect.x + rect.width {
                    break;
                }

                let hovered = self.is_bar_item_active(index);
                let (fg, bg) = bar_item_colors(item, self.style, hovered);
                paint_bar_label(
                    console,
                    x,
                    rect.y,
                    &label,
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
            if damage.intersects_rect(popup_rect) {
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

fn paint_bar_label(
    console: &mut Console,
    x: i64,
    y: i64,
    label: &str,
    item: &MenuBarItem,
    colors: MenuLabelPaint,
) {
    paint_labeled_text(console, x, y, label, &item.shortcut, item.enabled, colors);
}

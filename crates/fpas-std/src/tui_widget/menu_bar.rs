//! Host-managed horizontal menu bar painted in Rust from a Pascal item model.
//!
//! Spec: `docs/pascal/std/tui-app.md`

use crate::mouse_action_index;
use crate::{CommandId, Console, DamageRegion, UiMouse, ViewRect};

/// One declarative menu entry supplied from Pascal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuBarItem {
    /// Visible label text.
    pub label: String,
    /// When false, the entry is drawn disabled and ignores clicks.
    pub enabled: bool,
    /// Command dispatched through `OnCommand` on click, or `-1` when not clickable.
    pub command_id: i64,
}

/// CRT colors used while painting a menu bar widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuBarStyle {
    /// Normal bar background.
    pub bar_bg: u8,
    /// Normal bar foreground.
    pub bar_fg: u8,
    /// Background for the hovered enabled item.
    pub highlight_bg: u8,
    /// Foreground for the hovered enabled item.
    pub highlight_fg: u8,
    /// Foreground for disabled items.
    pub disabled_fg: u8,
}

impl Default for MenuBarStyle {
    fn default() -> Self {
        Self {
            bar_bg: 7,
            bar_fg: 0,
            highlight_bg: 0,
            highlight_fg: 7,
            disabled_fg: 8,
        }
    }
}

/// Result of routing one mouse event to a menu bar widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuBarMouseResult {
    /// The widget did not consume the event.
    Ignored,
    /// Hover highlight changed; caller should redraw the view.
    HoverChanged,
    /// A clickable item was activated.
    Command(CommandId),
}

/// Host-managed menu bar widget state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuBarWidget {
    pub items: Vec<MenuBarItem>,
    pub style: MenuBarStyle,
    hovered: Option<usize>,
}

impl MenuBarWidget {
    /// Creates a menu bar widget from Pascal-supplied model data.
    #[must_use]
    pub fn new(items: Vec<MenuBarItem>, style: MenuBarStyle) -> Self {
        Self {
            items,
            style,
            hovered: None,
        }
    }

    /// Replaces the menu model while preserving hover when possible.
    pub fn set_items(&mut self, items: Vec<MenuBarItem>) {
        self.hovered = self
            .hovered
            .filter(|index| *index < items.len() && items[*index].enabled);
        self.items = items;
    }

    /// Paint the menu bar clipped to `damage`.
    pub fn paint(self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
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

            let (fg, bg) = if !item.enabled {
                (self.style.disabled_fg, self.style.bar_bg)
            } else if self.hovered == Some(index) {
                (self.style.highlight_fg, self.style.highlight_bg)
            } else {
                (self.style.bar_fg, self.style.bar_bg)
            };

            console.write_text_at_crt(x, rect.y, &label, fg, bg);
            x += width;
        }
    }

    /// Route a mouse event within `rect` and update hover state when needed.
    pub fn handle_mouse(&mut self, rect: ViewRect, mouse: UiMouse) -> MenuBarMouseResult {
        if !rect.contains_point(mouse.x, mouse.y) {
            if self.hovered.take().is_some() {
                return MenuBarMouseResult::HoverChanged;
            }
            return MenuBarMouseResult::Ignored;
        }

        let item_index = item_index_at(self.items.as_slice(), rect, mouse.x);
        let down = mouse.action == mouse_action_index("Down");

        if down {
            if let Some(index) = item_index {
                let item = &self.items[index];
                if item.enabled && item.command_id >= 0 {
                    self.hovered = Some(index);
                    return MenuBarMouseResult::Command(CommandId(item.command_id));
                }
            }
            return MenuBarMouseResult::Ignored;
        }

        if self.hovered != item_index {
            self.hovered = item_index.filter(|index| self.items[*index].enabled);
            return MenuBarMouseResult::HoverChanged;
        }

        MenuBarMouseResult::Ignored
    }
}

fn item_index_at(items: &[MenuBarItem], rect: ViewRect, mouse_x: i64) -> Option<usize> {
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

fn item_display_width(item: &MenuBarItem) -> i64 {
    (item.label.chars().count() as i64).saturating_add(2)
}

fn clip_rect_to_damage(rect: ViewRect, damage: DamageRegion) -> Option<ViewRect> {
    match damage {
        DamageRegion::FullFrame => Some(rect),
        DamageRegion::Rect(dirty) => intersect_view_rects(rect, dirty),
    }
}

fn intersect_view_rects(left: ViewRect, right: ViewRect) -> Option<ViewRect> {
    let left_right = left.x.saturating_add(left.width.max(0));
    let left_bottom = left.y.saturating_add(left.height.max(0));
    let right_right = right.x.saturating_add(right.width.max(0));
    let right_bottom = right.y.saturating_add(right.height.max(0));

    let x = left.x.max(right.x);
    let y = left.y.max(right.y);
    let right = left_right.min(right_right);
    let bottom = left_bottom.min(right_bottom);

    if right <= x || bottom <= y {
        return None;
    }

    Some(ViewRect {
        x,
        y,
        width: right - x,
        height: bottom - y,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::Console;
    use fpas_bytecode::SourceLocation;

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn menu_bar_paints_labels() {
        let mut console = Console::new();
        console.assign_crt().unwrap();
        console.begin_tui_paint(DamageRegion::FullFrame);

        MenuBarWidget::new(
            vec![
                MenuBarItem {
                    label: "File".into(),
                    enabled: true,
                    command_id: 1,
                },
                MenuBarItem {
                    label: "Help".into(),
                    enabled: true,
                    command_id: 2,
                },
            ],
            MenuBarStyle::default(),
        )
        .paint(
            &mut console,
            ViewRect {
                x: 0,
                y: 0,
                width: 20,
                height: 1,
            },
            DamageRegion::FullFrame,
        );

        console.finish_tui_paint(loc()).unwrap();
        assert_eq!(console.test_cell(2, 1), ('F', 0, 7));
        assert_eq!(console.test_cell(8, 1), ('H', 0, 7));
    }

    #[test]
    fn menu_bar_click_dispatches_command() {
        let mut widget = MenuBarWidget::new(
            vec![MenuBarItem {
                label: "Quit".into(),
                enabled: true,
                command_id: 99,
            }],
            MenuBarStyle::default(),
        );
        let rect = ViewRect {
            x: 0,
            y: 0,
            width: 10,
            height: 1,
        };
        let result = widget.handle_mouse(
            rect,
            UiMouse::new(mouse_action_index("Down"), 1, 2, 0, Default::default()),
        );
        assert_eq!(result, MenuBarMouseResult::Command(CommandId(99)));
    }
}

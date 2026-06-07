//! Host-managed horizontal menu bar painted in Rust from a Pascal item model.
//!
//! Spec: `docs/pascal/std/tui-app.md`

use crate::key_event::{ConsoleKeyEvent, key_kind_index};
use crate::mouse_action_index;
use crate::{CommandId, Console, DamageRegion, UiMouse, ViewRect};

/// One declarative menu entry supplied from Pascal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuBarItem {
    /// Visible label text.
    pub label: String,
    /// Alt+letter shortcut (single character, case-insensitive). Empty means none.
    pub shortcut: String,
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
    /// Foreground for the Alt shortcut letter in normal state (Turbo Pascal red).
    pub accel_fg: u8,
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
            accel_fg: 4,
            highlight_bg: 0,
            highlight_fg: 7,
            disabled_fg: 8,
        }
    }
}

/// Result of routing one mouse or keyboard event to a menu bar widget.
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

            let hovered = self.hovered == Some(index);
            let (fg, bg) = if !item.enabled {
                (self.style.disabled_fg, self.style.bar_bg)
            } else if hovered {
                (self.style.highlight_fg, self.style.highlight_bg)
            } else {
                (self.style.bar_fg, self.style.bar_bg)
            };

            paint_menu_label(console, x, rect.y, item, MenuLabelPaint {
                fg,
                bg,
                accel_fg: self.style.accel_fg,
                hovered,
            });
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

    /// Route Alt+letter shortcuts and F10 menu activation.
    pub fn handle_key(&mut self, key: &ConsoleKeyEvent) -> MenuBarMouseResult {
        if key.kind == key_kind_index("F10") && !key.ctrl && !key.alt && !key.meta {
            return self.activate_first_item();
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

        let command_id = self.items[index].command_id;
        if command_id >= 0 {
            self.hovered = Some(index);
            return MenuBarMouseResult::Command(CommandId(command_id));
        }

        if self.hovered == Some(index) {
            return MenuBarMouseResult::Ignored;
        }
        self.hovered = Some(index);
        MenuBarMouseResult::HoverChanged
    }

    fn activate_first_item(&mut self) -> MenuBarMouseResult {
        let index = self.items.iter().position(|item| item.enabled);
        let Some(index) = index else {
            return MenuBarMouseResult::Ignored;
        };
        if self.hovered == Some(index) {
            return MenuBarMouseResult::Ignored;
        }
        self.hovered = Some(index);
        MenuBarMouseResult::HoverChanged
    }
}

struct MenuLabelPaint {
    fg: u8,
    bg: u8,
    accel_fg: u8,
    hovered: bool,
}

fn paint_menu_label(
    console: &mut Console,
    x: i64,
    y: i64,
    item: &MenuBarItem,
    colors: MenuLabelPaint,
) {
    let label = format!(" {} ", item.label);
    let highlight_index = shortcut_highlight_index(&item.label, &item.shortcut);
    let mut col = x;
    for (index, ch) in label.chars().enumerate() {
        let cell_fg = if colors.hovered || !item.enabled {
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

fn shortcut_highlight_index(label: &str, shortcut: &str) -> Option<usize> {
    let shortcut = shortcut.chars().next()?;
    if !shortcut.is_ascii_alphabetic() {
        return None;
    }
    let inner = format!(" {} ", label);
    inner
        .char_indices()
        .find(|(_, ch)| ch.eq_ignore_ascii_case(&shortcut))
        .map(|(index, _)| index)
}

fn shortcut_letter(key: &ConsoleKeyEvent) -> Option<char> {
    if !key.alt || key.ctrl || key.meta || key.kind != key_kind_index("Character") {
        return None;
    }
    key.ch
        .is_ascii_alphabetic()
        .then_some(key.ch.to_ascii_lowercase())
}

fn item_matches_shortcut(item: &MenuBarItem, shortcut: char) -> bool {
    item.shortcut
        .chars()
        .next()
        .is_some_and(|letter| letter.eq_ignore_ascii_case(&shortcut))
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

    fn file_item() -> MenuBarItem {
        MenuBarItem {
            label: "File".into(),
            shortcut: "F".into(),
            enabled: true,
            command_id: -1,
        }
    }

    #[test]
    fn menu_bar_paints_shortcut_letter_in_accel_color() {
        let mut console = Console::new();
        console.assign_crt().unwrap();
        console.begin_tui_paint(DamageRegion::FullFrame);

        MenuBarWidget::new(vec![file_item()], MenuBarStyle::default()).paint(
            &mut console,
            ViewRect {
                x: 0,
                y: 0,
                width: 10,
                height: 1,
            },
            DamageRegion::FullFrame,
        );

        console.finish_tui_paint(loc()).unwrap();
        assert_eq!(console.test_cell(2, 1), ('F', 4, 7));
        assert_eq!(console.test_cell(3, 1), ('i', 0, 7));
    }

    #[test]
    fn menu_bar_alt_shortcut_highlights_item() {
        let mut widget = MenuBarWidget::new(vec![file_item()], MenuBarStyle::default());
        let key = ConsoleKeyEvent::new(key_kind_index("Character"), 'f', false, false, true, false);
        assert_eq!(widget.handle_key(&key), MenuBarMouseResult::HoverChanged);
        assert_eq!(
            widget.handle_key(&key),
            MenuBarMouseResult::Ignored,
            "second Alt+F should not redraw"
        );
    }

    #[test]
    fn menu_bar_alt_shortcut_dispatches_command() {
        let mut widget = MenuBarWidget::new(
            vec![MenuBarItem {
                label: "Quit".into(),
                shortcut: "Q".into(),
                enabled: true,
                command_id: 99,
            }],
            MenuBarStyle::default(),
        );
        let key = ConsoleKeyEvent::new(key_kind_index("Character"), 'q', false, false, true, false);
        assert_eq!(
            widget.handle_key(&key),
            MenuBarMouseResult::Command(CommandId(99))
        );
    }

    #[test]
    fn menu_bar_f10_activates_first_item() {
        let mut widget = MenuBarWidget::new(vec![file_item()], MenuBarStyle::default());
        let key = ConsoleKeyEvent::new(key_kind_index("F10"), '\0', false, false, false, false);
        assert_eq!(widget.handle_key(&key), MenuBarMouseResult::HoverChanged);
        assert_eq!(widget.handle_key(&key), MenuBarMouseResult::Ignored);
    }

    #[test]
    fn menu_bar_click_dispatches_command() {
        let mut widget = MenuBarWidget::new(
            vec![MenuBarItem {
                label: "Quit".into(),
                shortcut: String::new(),
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

//! Host-managed horizontal menu bar painted in Rust from a Pascal item model.
//!
//! Spec: `docs/pascal/std/tui-app.md`

use crate::key_event::{ConsoleKeyEvent, key_kind_index};
use crate::mouse_action_index;
use crate::{CommandId, Console, DamageRegion, UiMouse, ViewRect};

use super::menu_popup::{
    MenuPopupItem, MenuPopupRect, paint_popup, popup_alt_shortcut_index, popup_entry_at,
    popup_rect, popup_shortcut_index,
};

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
    /// Pull-down entries shown when this top-level item is activated.
    pub submenu: Vec<MenuPopupItem>,
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
    /// Hover or submenu state changed; caller should redraw affected regions.
    HoverChanged,
    /// A clickable item was activated.
    Command(CommandId),
}

/// Highlight and shortcut paint colors for one menu label row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tui_widget) struct MenuLabelPaint {
    pub fg: u8,
    pub bg: u8,
    pub accel_fg: u8,
    pub hovered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OpenSubmenu {
    bar_index: usize,
    entry_index: usize,
}

/// Host-managed menu bar widget state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuBarWidget {
    pub items: Vec<MenuBarItem>,
    pub style: MenuBarStyle,
    hovered: Option<usize>,
    open_submenu: Option<OpenSubmenu>,
    menu_active: bool,
}

impl MenuBarWidget {
    /// Creates a menu bar widget from Pascal-supplied model data.
    #[must_use]
    pub fn new(items: Vec<MenuBarItem>, style: MenuBarStyle) -> Self {
        Self {
            items,
            style,
            hovered: None,
            open_submenu: None,
            menu_active: false,
        }
    }

    /// Replaces the menu model while preserving hover when possible.
    pub fn set_items(&mut self, items: Vec<MenuBarItem>) {
        self.hovered = self
            .hovered
            .filter(|index| *index < items.len() && items[*index].enabled);
        if let Some(open) = self.open_submenu
            && open.bar_index >= items.len()
        {
            self.open_submenu = None;
            self.menu_active = false;
        }
        self.items = items;
    }

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
            || self
                .open_popup_rect(bar_rect)
                .is_some_and(|popup| popup.contains_view_point(view_mouse_x(mouse_x), view_mouse_y(mouse_y)))
    }

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
    pub fn paint_popup_overlay(
        &self,
        console: &mut Console,
        rect: ViewRect,
        damage: DamageRegion,
    ) {
        if let Some((popup, entries, selected)) = self.open_popup(rect) {
            let popup_rect = popup.as_view_rect();
            if intersects_damage(popup_rect, damage) {
                self.paint_popup(console, popup, entries, selected);
            }
        }
    }

    fn paint_popup(
        &self,
        console: &mut Console,
        popup: MenuPopupRect,
        entries: &[MenuPopupItem],
        selected: usize,
    ) {
        paint_popup(console, popup, entries, self.style, selected);
    }

    /// Route a mouse event within the menu bar and open popup regions.
    pub fn handle_mouse(&mut self, rect: ViewRect, mouse: UiMouse) -> MenuBarMouseResult {
        let (mouse_x, mouse_y) = view_mouse_coords(mouse);
        let down = mouse.action == mouse_action_index("Down");

        if let Some(open) = self.open_submenu {
            let Some(popup) = self.open_popup_rect(rect) else {
                return MenuBarMouseResult::Ignored;
            };
            if popup.contains_view_point(mouse_x, mouse_y) {
                if !down {
                    return MenuBarMouseResult::Ignored;
                }
                let entry_index =
                    popup_entry_at(popup, &self.items[open.bar_index].submenu, mouse_x, mouse_y);
                let Some(entry_index) = entry_index else {
                    return MenuBarMouseResult::Ignored;
                };
                let (command_id, enabled) = {
                    let entry = &self.items[open.bar_index].submenu[entry_index];
                    (entry.command_id, entry.enabled)
                };
                if !enabled || command_id < 0 {
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
            if entry.enabled && entry.command_id >= 0 {
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
            if entry.enabled && entry.command_id >= 0 {
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
        let first_enabled = self.items[index]
            .submenu
            .iter()
            .position(|entry| entry.enabled)
            .unwrap_or(0);
        self.hovered = Some(index);
        self.menu_active = true;
        self.open_submenu = Some(OpenSubmenu {
            bar_index: index,
            entry_index: first_enabled,
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
                .position(|entry| entry.enabled)
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
            if entries[index].enabled {
                open.entry_index = index;
                return;
            }
        }
    }

    fn is_bar_item_active(&self, index: usize) -> bool {
        self.hovered == Some(index)
            || self
                .open_submenu
                .is_some_and(|open| open.bar_index == index)
    }

    fn open_popup(&self, bar_rect: ViewRect) -> Option<(MenuPopupRect, &[MenuPopupItem], usize)> {
        let open = self.open_submenu?;
        let entries = &self.items[open.bar_index].submenu;
        if entries.is_empty() {
            return None;
        }
        let anchor_x = item_x_at(self.items.as_slice(), bar_rect, open.bar_index)?;
        let popup = popup_rect(anchor_x, bar_rect.y + bar_rect.height, entries);
        Some((popup, entries.as_slice(), open.entry_index))
    }

    fn open_popup_rect(&self, bar_rect: ViewRect) -> Option<MenuPopupRect> {
        self.open_popup(bar_rect).map(|(popup, _, _)| popup)
    }
}

fn view_mouse_x(mouse_x: i64) -> i64 {
    mouse_x.saturating_sub(1)
}

fn view_mouse_y(mouse_y: i64) -> i64 {
    mouse_y.saturating_sub(1)
}

fn view_mouse_coords(mouse: UiMouse) -> (i64, i64) {
    (view_mouse_x(mouse.x), view_mouse_y(mouse.y))
}

fn has_submenu(item: &MenuBarItem) -> bool {
    !item.submenu.is_empty()
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

pub(in crate::tui_widget) fn paint_bar_label(
    console: &mut Console,
    x: i64,
    y: i64,
    item: &MenuBarItem,
    colors: MenuLabelPaint,
) {
    let label = format!(" {} ", item.label);
    paint_labeled_text(console, x, y, &label, &item.shortcut, item.enabled, colors);
}

pub(in crate::tui_widget) fn paint_labeled_text(
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

pub(in crate::tui_widget) fn shortcut_highlight_index(
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

fn popup_shortcut_key_index(entries: &[MenuPopupItem], key: &ConsoleKeyEvent) -> Option<usize> {
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

fn item_x_at(items: &[MenuBarItem], rect: ViewRect, index: usize) -> Option<i64> {
    let mut x = rect.x;
    for (current, item) in items.iter().enumerate() {
        if current == index {
            return Some(x);
        }
        x = x.saturating_add(item_display_width(item));
    }
    None
}

fn item_display_width(item: &MenuBarItem) -> i64 {
    (item.label.chars().count() as i64).saturating_add(2)
}

fn intersects_damage(rect: ViewRect, damage: DamageRegion) -> bool {
    match damage {
        DamageRegion::FullFrame => true,
        DamageRegion::Rect(dirty) => intersect_view_rects(rect, dirty).is_some(),
    }
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
            submenu: vec![MenuPopupItem {
                label: "Exit".into(),
                shortcut: "X".into(),
                enabled: true,
                command_id: 1,
            }],
        }
    }

    fn bar_rect() -> ViewRect {
        ViewRect {
            x: 0,
            y: 0,
            width: 20,
            height: 1,
        }
    }

    #[test]
    fn menu_bar_paints_shortcut_letter_in_accel_color() {
        let mut console = Console::new();
        console.assign_crt().unwrap();
        console.begin_tui_paint(DamageRegion::FullFrame);

        MenuBarWidget::new(vec![file_item()], MenuBarStyle::default()).paint(
            &mut console,
            bar_rect(),
            DamageRegion::FullFrame,
        );

        console.finish_tui_paint(loc()).unwrap();
        assert_eq!(console.test_cell(2, 1), ('F', 4, 7));
        assert_eq!(console.test_cell(3, 1), ('i', 0, 7));
    }

    #[test]
    fn menu_bar_alt_shortcut_opens_submenu() {
        let mut widget = MenuBarWidget::new(vec![file_item()], MenuBarStyle::default());
        let key = ConsoleKeyEvent::new(key_kind_index("Character"), 'f', false, false, true, false);
        assert_eq!(widget.handle_key(&key), MenuBarMouseResult::HoverChanged);
        assert_eq!(widget.damage_rects(bar_rect()).len(), 2);
    }

    #[test]
    fn menu_bar_submenu_enter_dispatches_command() {
        let mut widget = MenuBarWidget::new(vec![file_item()], MenuBarStyle::default());
        let key = ConsoleKeyEvent::new(key_kind_index("Character"), 'f', false, false, true, false);
        let _ = widget.handle_key(&key);
        let enter = ConsoleKeyEvent::new(key_kind_index("Enter"), '\0', false, false, false, false);
        assert_eq!(
            widget.handle_key(&enter),
            MenuBarMouseResult::Command(CommandId(1))
        );
        assert_eq!(widget.damage_rects(bar_rect()).len(), 1);
    }

    #[test]
    fn menu_bar_f10_opens_first_submenu() {
        let mut widget = MenuBarWidget::new(vec![file_item()], MenuBarStyle::default());
        let key = ConsoleKeyEvent::new(key_kind_index("F10"), '\0', false, false, false, false);
        assert_eq!(widget.handle_key(&key), MenuBarMouseResult::HoverChanged);
        assert_eq!(widget.damage_rects(bar_rect()).len(), 2);
    }

    #[test]
    fn menu_bar_submenu_click_dispatches_command() {
        let mut widget = MenuBarWidget::new(vec![file_item()], MenuBarStyle::default());
        let key = ConsoleKeyEvent::new(key_kind_index("Character"), 'f', false, false, true, false);
        let _ = widget.handle_key(&key);

        // Popup entry row: view y=2 → console y=3 (one-based).
        let result = widget.handle_mouse(
            bar_rect(),
            UiMouse::new(mouse_action_index("Down"), 1, 2, 3, Default::default()),
        );
        assert_eq!(result, MenuBarMouseResult::Command(CommandId(1)));
        assert_eq!(widget.damage_rects(bar_rect()).len(), 1);
    }

    #[test]
    fn menu_bar_click_dispatches_command() {
        let mut widget = MenuBarWidget::new(
            vec![MenuBarItem {
                label: "Quit".into(),
                shortcut: String::new(),
                enabled: true,
                command_id: 99,
                submenu: Vec::new(),
            }],
            MenuBarStyle::default(),
        );
        let result = widget.handle_mouse(
            bar_rect(),
            UiMouse::new(mouse_action_index("Down"), 1, 2, 1, Default::default()),
        );
        assert_eq!(result, MenuBarMouseResult::Command(CommandId(99)));
    }
}

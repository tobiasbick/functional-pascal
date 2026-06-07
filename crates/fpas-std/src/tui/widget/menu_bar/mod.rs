//! Host-managed horizontal menu bar painted in Rust from a Pascal item model.
//!
//! Spec: `docs/pascal/std/tui-app.md`

mod geometry;
mod input;
mod paint;
mod types;

#[cfg(test)]
mod tests;

pub use types::{MenuBarItem, MenuBarMouseResult, MenuBarStyle};

pub(in crate::tui::widget) use paint::shortcut_highlight_index;
pub(in crate::tui::widget) use types::MenuLabelPaint;

use types::OpenSubmenu;

/// Host-managed menu bar widget state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuBarWidget {
    /// Declarative top-level menu entries from Pascal.
    pub items: Vec<MenuBarItem>,
    /// CRT colors used while painting the bar and popups.
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
}

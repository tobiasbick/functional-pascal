//! Host-managed horizontal menu bar painted in Rust from a Pascal item model.
//!
//! Spec: `docs/pascal/std/tui/app/README.md`

mod geometry;
mod input;
mod paint;
mod types;

#[cfg(test)]
mod tests;

pub use super::menu_style::MenuBarStyle;
pub use types::{MenuBarItem, MenuBarMouseResult, MenuBarState};

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

    /// Returns a read-only snapshot of hover, activation, and submenu state.
    ///
    /// **Documentation:** `docs/pascal/std/tui/app/README.md` (`Application.QueryMenuBarState`)
    #[must_use]
    pub fn query_state(&self) -> MenuBarState {
        let (submenu_open, submenu_bar_index, selected_entry) = match self.open_submenu {
            Some(open) => (
                true,
                i64::try_from(open.bar_index).unwrap_or(-1),
                i64::try_from(open.entry_index).unwrap_or(-1),
            ),
            None => (false, -1, -1),
        };
        MenuBarState {
            menu_active: self.menu_active,
            hovered_index: self
                .hovered
                .and_then(|index| i64::try_from(index).ok())
                .unwrap_or(-1),
            submenu_open,
            submenu_bar_index,
            selected_entry,
        }
    }
}

//! Menu bar and status line session state.

use super::*;

impl TurboVisionSession {
    pub fn menu_item_command_id(
        &self,
        handle: u32,
        menu_index: usize,
        item_index: usize,
    ) -> Option<u16> {
        let state = self.menu_bars.get(&handle)?;
        let menu = state.menus.get(menu_index)?;
        let item = menu.items.get(item_index)?;
        if item.command_id == 0 {
            None
        } else {
            Some(item.command_id)
        }
    }

    /// Stores menu bar data for a registry handle.
    pub fn insert_menu_bar(&mut self, handle: u32, state: TuiMenuBarState) {
        self.menu_bars.insert(handle, state);
    }

    /// Replaces menu bar menus (`MenuBar.SetMenus`).
    pub fn set_menu_bar_menus(
        &mut self,
        handle: u32,
        menus: Vec<TurboVisionMenu>,
    ) -> Result<(), ()> {
        let Some(state) = self.menu_bars.get_mut(&handle) else {
            return Err(());
        };
        state.menus = menus;
        Ok(())
    }

    /// Stores status line data for a registry handle.
    pub fn insert_status_line(&mut self, handle: u32, state: TuiStatusLineState) {
        self.status_lines.insert(handle, state);
    }

    /// Replaces status line items (`StatusLine.SetItems`).
    pub fn set_status_line_items(
        &mut self,
        handle: u32,
        items: Vec<TurboVisionStatusItem>,
    ) -> Result<(), ()> {
        let Some(state) = self.status_lines.get_mut(&handle) else {
            return Err(());
        };
        state.items = items;
        Ok(())
    }

    /// Returns the attached menu bar handle, if any.
    #[must_use]
    pub fn attached_menu_bar(&self) -> Option<u32> {
        self.attached_menu_bar
    }

    /// Returns the attached status line handle, if any.
    #[must_use]
    pub fn attached_status_line(&self) -> Option<u32> {
        self.attached_status_line
    }

    /// Marks a menu bar as application chrome.
    pub fn set_attached_menu_bar(&mut self, handle: u32) {
        self.attached_menu_bar = Some(handle);
    }

    /// Marks a status line as application chrome.
    pub fn set_attached_status_line(&mut self, handle: u32) {
        self.attached_status_line = Some(handle);
    }

    /// Snapshot of the attached menu bar, if set.
    #[must_use]
    pub fn attached_menu_bar_snapshot(&self) -> Option<&TuiMenuBarState> {
        self.attached_menu_bar
            .and_then(|handle| self.menu_bars.get(&handle))
    }

    /// Snapshot of the attached status line, if set.
    #[must_use]
    pub fn attached_status_line_snapshot(&self) -> Option<&TuiStatusLineState> {
        self.attached_status_line
            .and_then(|handle| self.status_lines.get(&handle))
    }
}

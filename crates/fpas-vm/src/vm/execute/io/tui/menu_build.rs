//! Turbo Vision menu construction from FPAS menu records.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use crate::vm::shared::TurboVisionMenu;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};

use super::tv_geometry::turbo_rect;

/// Build an upstream Turbo Vision menu from a stored FPAS menu snapshot.
pub(in crate::vm::execute::io::tui) fn build_upstream_menu(menu: &TurboVisionMenu) -> Menu {
    let items = menu
        .items
        .iter()
        .map(|item| {
            if item.command_id == 0 {
                MenuItem::separator()
            } else {
                MenuItem::new(&item.text, item.command_id, 0, 0)
            }
        })
        .collect();
    Menu::from_items(items)
}

/// Build an upstream Turbo Vision menu bar from stored FPAS state.
pub(in crate::vm::execute::io::tui) fn build_menu_bar(
    bounds: turbo_vision::core::geometry::Rect,
    menus: &[TurboVisionMenu],
) -> MenuBar {
    let mut menu_bar = MenuBar::new(bounds);
    for menu in menus {
        menu_bar.add_submenu(SubMenu::new(&menu.title, build_upstream_menu(menu)));
    }
    menu_bar
}

/// Build an upstream menu bar from a menu-bar snapshot.
pub(in crate::vm::execute::io::tui) fn build_menu_bar_from_snapshot(
    bounds: crate::vm::shared::TurboVisionRect,
    menus: &[TurboVisionMenu],
) -> MenuBar {
    build_menu_bar(turbo_rect(bounds), menus)
}

#[cfg(test)]
mod tests {
    use super::build_upstream_menu;
    use crate::vm::shared::{TurboVisionMenu, TurboVisionMenuItem};
    use turbo_vision::core::menu_data::MenuItem;

    #[test]
    fn turbo_vision_menu_builder_maps_separator_and_commands() {
        let menu = build_upstream_menu(&TurboVisionMenu {
            title: "~F~ile".into(),
            items: vec![
                TurboVisionMenuItem {
                    text: "~O~pen".into(),
                    command_id: 100,
                },
                TurboVisionMenuItem {
                    text: "-".into(),
                    command_id: 0,
                },
                TurboVisionMenuItem {
                    text: "E~x~it".into(),
                    command_id: 4,
                },
            ],
        });

        assert_eq!(menu.len(), 3);
        assert!(matches!(menu.items[0], MenuItem::Regular { .. }));
        assert!(matches!(menu.items[1], MenuItem::Separator));
        assert_eq!(menu.items[2].command(), Some(4));
    }
}

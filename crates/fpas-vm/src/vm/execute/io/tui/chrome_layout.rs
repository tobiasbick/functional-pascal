//! Live-terminal layout for Turbo Vision menu bar and status line.
//!
//! Matches upstream `Application::handle_redraw` so chrome spans the full
//! terminal before the first frame, not only after a resize event.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use turbo_vision::core::geometry::Rect;
use turbo_vision::views::View;
use turbo_vision::views::{menu_bar::MenuBar, status_line::StatusLine};

/// Stretch the menu bar to the full terminal width on the top row (TV row 0).
pub(in crate::vm::execute::io::tui) fn layout_menu_bar_for_terminal(
    menu_bar: &mut MenuBar,
    terminal_width: i16,
) {
    let height = menu_bar.bounds().height();
    let target = Rect::new(0, 0, terminal_width, height);
    if menu_bar.bounds() != target {
        menu_bar.set_bounds(target);
    }
}

/// Pin the status line to the bottom row and stretch it to the full terminal width.
pub(in crate::vm::execute::io::tui) fn layout_status_line_for_terminal(
    status_line: &mut StatusLine,
    terminal_width: i16,
    terminal_height: i16,
) {
    let bounds = status_line.bounds();
    let top = terminal_height - bounds.height();
    if bounds.a.x == 0 && bounds.a.y == top && bounds.width() == terminal_width {
        return;
    }
    status_line.set_bounds(Rect::new(0, top, terminal_width, terminal_height));
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_vision::core::command::CM_QUIT;
    use turbo_vision::core::geometry::Rect;
    use turbo_vision::core::menu_data::Menu;
    use turbo_vision::views::View;
    use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
    use turbo_vision::views::status_line::{StatusItem, StatusLine};

    #[test]
    fn layout_menu_bar_stretches_to_terminal_width() {
        let mut menu_bar = MenuBar::new(Rect::new(0, 0, 80, 1));
        menu_bar.add_submenu(SubMenu::new("File", Menu::new()));
        layout_menu_bar_for_terminal(&mut menu_bar, 120);
        assert_eq!(menu_bar.bounds(), Rect::new(0, 0, 120, 1));
    }

    #[test]
    fn layout_status_line_pins_to_bottom_and_stretches_width() {
        let status_line = StatusLine::new(
            Rect::new(0, 24, 80, 25),
            vec![StatusItem::new("Quit", 0, CM_QUIT)],
        );
        let mut status_line = status_line;
        layout_status_line_for_terminal(&mut status_line, 120, 30);
        assert_eq!(status_line.bounds(), Rect::new(0, 29, 120, 30));
    }
}

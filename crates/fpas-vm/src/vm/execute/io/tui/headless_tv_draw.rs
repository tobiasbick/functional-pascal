//! Headless turbo-vision draw path: upstream `draw` into a memory backend, then CRT export.
//!
//! **Documentation:** `docs/refactor/tui-bridge/done/04-headless-test-util.md`

use super::chrome_layout::{layout_menu_bar_for_terminal, layout_status_line_for_terminal};
use super::menu_build::build_menu_bar_from_snapshot;
use super::tv_headless_backend::{HeadlessTvEventInbox, TvHeadlessBackend};
use crate::vm::Worker;
use fpas_bytecode::SourceLocation;
use fpas_std::Console;
use std::time::Duration;
use turbo_vision::core::command_set;
use turbo_vision::core::draw::Cell;
use turbo_vision::core::event::{Event, EventType, MB_LEFT_BUTTON};
use turbo_vision::core::geometry::Point;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::TvColor;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::View;
use turbo_vision::views::desktop::Desktop;
use turbo_vision::views::menu_bar::MenuBar;
use turbo_vision::views::status_line::StatusLine;

/// Turbo Vision session used for headless paint (no crossterm).
pub(in crate::vm) struct HeadlessTvApp {
    terminal: Terminal,
    desktop: Desktop,
    menu_bar: Option<MenuBar>,
    status_line: Option<StatusLine>,
    event_inbox: HeadlessTvEventInbox,
}

impl HeadlessTvApp {
    fn new(width: u16, height: u16) -> turbo_vision::core::error::Result<Self> {
        let (backend, event_inbox) = TvHeadlessBackend::new(width, height);
        let terminal = Terminal::with_backend(Box::new(backend))?;
        let (width, height) = terminal.size();
        command_set::init_command_set();
        let mut desktop = Desktop::new(Rect::new(0, 0, width, height));
        desktop.init_palette_chain();
        Ok(Self {
            terminal,
            desktop,
            menu_bar: None,
            status_line: None,
            event_inbox,
        })
    }

    /// Queue a left mouse down at desktop coordinates for headless test input.
    pub(in crate::vm::execute::io::tui) fn push_mouse_down(&self, x: i16, y: i16) {
        self.event_inbox.push(Event::mouse(
            EventType::MouseDown,
            Point::new(x, y),
            MB_LEFT_BUTTON,
            false,
        ));
    }

    /// Poll one queued input event and dispatch it through the desktop view tree.
    pub(in crate::vm::execute::io::tui) fn dispatch_next_input_event(
        &mut self,
    ) -> std::io::Result<Option<Event>> {
        let Some(mut event) = self.terminal.poll_event(Duration::ZERO)? else {
            return Ok(None);
        };
        self.desktop.handle_event(&mut event);
        Ok(Some(event))
    }

    fn update_desktop_bounds(&mut self) {
        let (width, height) = self.terminal.size();
        let mut desktop_bounds = Rect::new(0, 0, width, height);

        if let Some(ref menu_bar) = self.menu_bar {
            desktop_bounds.a.y += menu_bar.bounds().height();
        } else {
            desktop_bounds.a.y += 1;
        }

        if let Some(ref status_line) = self.status_line {
            desktop_bounds.b.y -= status_line.bounds().height();
        } else {
            desktop_bounds.b.y -= 1;
        }

        self.desktop.set_bounds(desktop_bounds);
    }

    fn draw(&mut self) {
        self.desktop.draw(&mut self.terminal);
        if let Some(ref mut menu_bar) = self.menu_bar {
            menu_bar.draw(&mut self.terminal);
        }
        if let Some(ref mut status_line) = self.status_line {
            status_line.draw(&mut self.terminal);
        }
        self.desktop.update_cursor(&mut self.terminal);
    }

    pub(in crate::vm::execute::io::tui) fn desktop_mut(&mut self) -> &mut Desktop {
        &mut self.desktop
    }
}

impl Worker {
    /// Repaint the headless desktop using upstream turbo-vision `draw`.
    ///
    /// When `rebuild` is false, the existing desktop tree is kept and only redrawn.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_paint_headless_desktop(
        &mut self,
        _line: SourceLocation,
        rebuild: bool,
    ) {
        let width = self.with_console(|console| console.screen_width() as u16);
        let height = self.with_console(|console| console.screen_height() as u16);
        let needs_new = self.headless_tv_app.as_ref().is_none_or(|app| {
            let (app_w, app_h) = app.terminal.size();
            i64::from(app_w) != i64::from(width) || i64::from(app_h) != i64::from(height)
        });
        if needs_new {
            self.turbo_vision_shutdown_headless_app();
            if let Ok(app) = HeadlessTvApp::new(width, height) {
                self.headless_tv_app = Some(app);
            }
        }

        let mut app_slot = self.headless_tv_app.take();
        let Some(app) = app_slot.as_mut() else {
            self.headless_tv_app = app_slot;
            return;
        };

        self.turbo_vision_sync_chrome_to_headless_app(app);
        if rebuild {
            while app.desktop.child_count() > 0 {
                app.desktop.remove_child(0);
            }
            self.turbo_vision_populate_desktop_on(&mut app.desktop);
        }
        app.draw();

        let terminal = &app.terminal;
        self.with_console(|console| export_terminal_buffer_to_console(terminal, console));
        self.headless_tv_app = app_slot;
    }

    /// Release the headless turbo-vision session.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_shutdown_headless_app(&mut self) {
        if let Some(mut app) = self.headless_tv_app.take() {
            let _ = app.terminal.shutdown();
        }
    }

    fn turbo_vision_sync_chrome_to_headless_app(&self, app: &mut HeadlessTvApp) {
        let (terminal_width, terminal_height) = app.terminal.size();

        if let Some(menu_bar) = self.turbo_vision_menu_bar_snapshot() {
            let mut menu_bar = build_menu_bar_from_snapshot(menu_bar.bounds, &menu_bar.menus);
            layout_menu_bar_for_terminal(&mut menu_bar, terminal_width);
            app.menu_bar = Some(menu_bar);
        } else {
            app.menu_bar = None;
        }

        if let Some(status_line) = self.turbo_vision_status_line_snapshot() {
            let mut status_line = super::tv_views::build_status_line(status_line);
            layout_status_line_for_terminal(&mut status_line, terminal_width, terminal_height);
            app.status_line = Some(status_line);
        } else {
            app.status_line = None;
        }

        app.update_desktop_bounds();
    }
}

fn export_terminal_buffer_to_console(terminal: &Terminal, console: &mut Console) {
    console.clear_headless_screen();
    let buffer = terminal.buffer();
    for (row, line) in buffer.iter().enumerate() {
        for (col, cell) in line.iter().enumerate() {
            let (fg, bg) = tv_attr_to_console_colors(cell);
            console.paint_headless_cell(col as u16 + 1, row as u16 + 1, cell.ch, fg, bg);
        }
    }
}

fn tv_attr_to_console_colors(cell: &Cell) -> (u8, u8) {
    (
        tv_color_to_console_index(cell.attr.fg),
        tv_color_to_console_index(cell.attr.bg),
    )
}

fn tv_color_to_console_index(color: TvColor) -> u8 {
    match color {
        TvColor::Rgb { r, g, b } => TvColor::from_rgb(r, g, b).to_index(),
        other => other.to_index(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::chrome_layout::layout_status_line_for_terminal;
    use super::*;
    use turbo_vision::core::geometry::Rect;
    use turbo_vision::core::menu_data::Menu;
    use turbo_vision::views::dialog::Dialog;
    use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
    use turbo_vision::views::status_line::{StatusItem, StatusLine};

    fn find_char(buffer: &[Vec<Cell>], ch: char) -> Option<(usize, usize)> {
        for (row, line) in buffer.iter().enumerate() {
            for (col, cell) in line.iter().enumerate() {
                if cell.ch == ch {
                    return Some((col, row));
                }
            }
        }
        None
    }

    #[test]
    fn headless_tv_app_draws_menu_title_on_screen() {
        let mut app = HeadlessTvApp::new(60, 20).expect("headless app");
        let mut menu_bar = MenuBar::new(Rect::new(0, 0, 60, 1));
        menu_bar.add_submenu(SubMenu::new("OLD", Menu::new()));
        app.menu_bar = Some(menu_bar);
        app.update_desktop_bounds();
        app.draw();

        assert_eq!(
            find_char(app.terminal.buffer(), 'O'),
            Some((2, 0)),
            "menu title should paint at TV column 2, row 0"
        );
    }

    #[test]
    fn headless_tv_app_draws_status_text_on_bottom_row() {
        let mut app = HeadlessTvApp::new(60, 25).expect("headless app");
        let mut status_line =
            StatusLine::new(Rect::new(0, 24, 60, 25), vec![StatusItem::new("OLD", 0, 0)]);
        layout_status_line_for_terminal(&mut status_line, 60, 25);
        app.status_line = Some(status_line);
        app.update_desktop_bounds();
        app.draw();

        assert_eq!(
            find_char(app.terminal.buffer(), 'O'),
            Some((1, 24)),
            "status text should paint on bottom TV row"
        );
    }

    #[test]
    fn headless_tv_app_draws_chrome_menu_titles() {
        let mut app = HeadlessTvApp::new(60, 12).expect("headless app");
        let mut menu_bar = MenuBar::new(Rect::new(0, 0, 60, 1));
        menu_bar.add_submenu(SubMenu::new("FILE", Menu::new()));
        menu_bar.add_submenu(SubMenu::new("EDIT", Menu::new()));
        app.menu_bar = Some(menu_bar);
        app.update_desktop_bounds();
        app.draw();

        let buffer = app.terminal.buffer();
        assert_eq!(find_char(buffer, 'F'), Some((2, 0)));
        assert_eq!(find_char(buffer, 'E'), Some((5, 0)));
    }

    #[test]
    fn headless_tv_app_draws_static_text_in_dialog() {
        let mut app = HeadlessTvApp::new(60, 20).expect("headless app");
        app.update_desktop_bounds();
        let mut dialog_view = Dialog::new_modal(Rect::from_coords(2, 1, 30, 8), "Host");
        dialog_view.add(Box::new(turbo_vision::views::static_text::StaticText::new(
            Rect::from_coords(3, 2, 20, 1),
            "OLD",
        )));
        app.desktop.add(dialog_view);
        app.draw();

        assert_eq!(
            find_char(app.terminal.buffer(), 'O'),
            Some((6, 5)),
            "static text O in modal dialog (TV buffer coords)"
        );
    }

    fn find_substring(buffer: &[Vec<Cell>], text: &str) -> Option<(usize, usize)> {
        for (row, line) in buffer.iter().enumerate() {
            let row_text: String = line.iter().map(|cell| cell.ch).collect();
            if let Some(col) = row_text.find(text) {
                return Some((col, row));
            }
        }
        None
    }

    #[test]
    fn headless_tv_app_draws_static_text_in_window() {
        use super::super::bridged_static_text::BridgedStaticText;
        use turbo_vision::views::window::Window;

        let mut app = HeadlessTvApp::new(60, 20).expect("headless app");
        app.update_desktop_bounds();
        let mut window = Window::new(Rect::from_coords(8, 4, 30, 10), "Live");
        window.add(Box::new(BridgedStaticText::new(
            Rect::from_coords(4, 3, 10, 1),
            "LIVE",
        )));
        app.desktop.add(Box::new(window));
        app.draw();

        let buffer = app.terminal.buffer();
        assert_eq!(
            find_substring(buffer, "LIVE"),
            Some((31, 9)),
            "static text LIVE in desktop window (TV buffer coords)"
        );
    }

    #[test]
    fn headless_tv_app_draws_window_above_older_dialog() {
        use super::super::bridged_static_text::BridgedStaticText;
        use turbo_vision::views::window::Window;

        let mut app = HeadlessTvApp::new(60, 20).expect("headless app");
        app.update_desktop_bounds();
        let host_dialog = Dialog::new_modal(Rect::from_coords(2, 1, 24, 8), "Host");
        app.desktop.add(host_dialog);
        let mut window = Window::new(Rect::from_coords(8, 4, 30, 10), "Live");
        window.add(Box::new(BridgedStaticText::new(
            Rect::from_coords(4, 3, 10, 1),
            "LIVE",
        )));
        app.desktop.add(Box::new(window));
        app.draw();

        assert_eq!(
            find_substring(app.terminal.buffer(), "LIVE"),
            Some((31, 9)),
            "window static text paints above an older desktop dialog"
        );
    }

    #[test]
    fn headless_tv_app_draws_dialog_title_and_checkbox_marker() {
        let mut app = HeadlessTvApp::new(60, 20).expect("headless app");
        app.update_desktop_bounds();
        let mut dialog_view = Dialog::new_modal(Rect::from_coords(2, 1, 30, 8), "OLD");
        let check_box =
            turbo_vision::views::checkbox::CheckBox::new(Rect::from_coords(3, 2, 20, 1), "opt");
        dialog_view.add(Box::new(check_box));
        app.desktop.add(dialog_view);
        app.draw();

        let buffer = app.terminal.buffer();
        assert_eq!(find_char(buffer, 'O'), Some((16, 2)), "dialog title O");
        assert_eq!(find_char(buffer, '['), Some((4, 2)), "checkbox marker");
    }
}

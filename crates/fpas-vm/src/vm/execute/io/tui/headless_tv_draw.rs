//! Headless turbo-vision draw path: upstream `draw` into a memory backend, then CRT export.
//!
//! **Documentation:** `docs/refactor/tui-bridge/03-headless-test-util.md`

use super::chrome_layout::{layout_menu_bar_for_terminal, layout_status_line_for_terminal};
use super::menu_build::build_menu_bar_from_snapshot;
use super::tv_headless_backend::TvHeadlessBackend;
use crate::vm::Worker;
use fpas_bytecode::SourceLocation;
use fpas_std::Console;
use turbo_vision::core::command_set;
use turbo_vision::core::draw::Cell;
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
}

impl HeadlessTvApp {
    fn new(width: u16, height: u16) -> turbo_vision::core::error::Result<Self> {
        let backend = TvHeadlessBackend::new(width, height);
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
        })
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
}

impl Worker {
    /// Repaint the headless desktop using upstream turbo-vision `draw` (spike; not wired).
    #[allow(dead_code)]
    pub(in crate::vm::execute::io::tui) fn turbo_vision_paint_headless_desktop_via_tv_draw(
        &mut self,
        _line: SourceLocation,
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
        while app.desktop.child_count() > 0 {
            app.desktop.remove_child(0);
        }
        self.turbo_vision_populate_desktop_on(&mut app.desktop);
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
            if cell.ch == ' ' {
                continue;
            }
            let (fg, bg) = tv_attr_to_console_colors(cell);
            console.paint_headless_cell(col as u16 + 1, row as u16 + 1, cell.ch, fg, bg);
        }
    }
}

fn tv_attr_to_console_colors(cell: &Cell) -> (u8, u8) {
    let fg = match cell.attr.fg {
        TvColor::Rgb { .. } => 7,
        other => other.to_index(),
    };
    let bg = match cell.attr.bg {
        TvColor::Rgb { .. } => 0,
        other => other.to_index(),
    };
    (fg, bg)
}

//! Try-2 menu bar and status line chrome (`MenuBar.New`, `Application.SetMenuBar`, …).
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::app::try2_ensure_live_app;
use super::headless::try2_ensure_headless_app;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::execute::io::tui::chrome_layout::{
    layout_menu_bar_for_terminal, layout_status_line_for_terminal,
};
use crate::vm::execute::io::tui::try2::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::try2::session::{Try2MenuBarState, Try2StatusLineState};
use crate::vm::execute::io::tui::tv_geometry::{state_rect, turbo_rect};
use crate::vm::shared::{TurboVisionMenu, TurboVisionStatusItem};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::status_line::{StatusItem, StatusLine};

/// Creates a menu bar handle (`MenuBar.New`).
pub(in crate::vm::execute::io::tui::try2) fn try2_menu_bar_new(
    worker: &mut Worker,
    bounds: Rect,
    menus: Vec<TurboVisionMenu>,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.try2.is_open() {
        return Err(try2_session_closed_error(line));
    }

    let handle = worker.try2.registry.allocate(0, ViewKind::MenuBar);
    worker.try2.insert_menu_bar(
        handle,
        Try2MenuBarState {
            bounds: state_rect(bounds),
            menus,
        },
    );
    Ok(handle)
}

/// Creates a status line handle (`StatusLine.New`).
pub(in crate::vm::execute::io::tui::try2) fn try2_status_line_new(
    worker: &mut Worker,
    bounds: Rect,
    items: Vec<TurboVisionStatusItem>,
    line: SourceLocation,
) -> Result<u32, VmError> {
    if !worker.try2.is_open() {
        return Err(try2_session_closed_error(line));
    }

    let handle = worker.try2.registry.allocate(0, ViewKind::StatusLine);
    worker.try2.insert_status_line(
        handle,
        Try2StatusLineState {
            bounds: state_rect(bounds),
            items,
        },
    );
    Ok(handle)
}

/// Attaches a menu bar to the application session (`Application.SetMenuBar`).
pub(in crate::vm::execute::io::tui) fn try2_set_menu_bar(
    worker: &mut Worker,
    menu_bar_handle: u32,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .try2
        .registry
        .require(menu_bar_handle, ViewKind::MenuBar)
        .map_err(|error| menu_bar_error(error, line))?;

    if worker.try2.attached_menu_bar().is_some() {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Application already has a menu bar on the try-2 path",
            "Call `Application.SetMenuBar` only once per session.",
            line,
        ));
    }

    worker.try2.set_attached_menu_bar(menu_bar_handle);
    try2_sync_chrome_to_app(worker, line)
}

/// Attaches a status line to the application session (`Application.SetStatusLine`).
pub(in crate::vm::execute::io::tui) fn try2_set_status_line(
    worker: &mut Worker,
    status_line_handle: u32,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker
        .try2
        .registry
        .require(status_line_handle, ViewKind::StatusLine)
        .map_err(|error| status_line_error(error, line))?;

    if worker.try2.attached_status_line().is_some() {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Application already has a status line on the try-2 path",
            "Call `Application.SetStatusLine` only once per session.",
            line,
        ));
    }

    worker.try2.set_attached_status_line(status_line_handle);
    try2_sync_chrome_to_app(worker, line)
}

/// Applies stored menu bar and status line snapshots to the live or headless app.
pub(in crate::vm::execute::io::tui::try2) fn try2_sync_chrome_to_app(
    worker: &mut Worker,
    line: SourceLocation,
) -> Result<(), VmError> {
    if worker.try2.attached_menu_bar().is_none() && worker.try2.attached_status_line().is_none() {
        return Ok(());
    }

    let menu_bar = worker
        .try2
        .attached_menu_bar_snapshot()
        .map(|snapshot| build_try2_menu_bar(snapshot.bounds, &snapshot.menus));
    let status_line = worker
        .try2
        .attached_status_line_snapshot()
        .map(|snapshot| build_try2_status_line(snapshot.bounds, &snapshot.items));

    if worker.with_tui(|tui| tui.session.is_headless()) {
        try2_ensure_headless_app(worker, line)?;
        let Some(app) = worker.headless_tv_app.as_mut() else {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Headless Turbo Vision session is not initialized",
                "Call `Application.OpenForTest` before setting chrome.",
                line,
            ));
        };

        let (terminal_width, terminal_height) = app.terminal_size();
        let menu_bar = menu_bar.map(|mut bar| {
            layout_menu_bar_for_terminal(&mut bar, terminal_width);
            bar
        });
        let status_line = status_line.map(|mut line| {
            layout_status_line_for_terminal(&mut line, terminal_width, terminal_height);
            line
        });
        app.replace_chrome(menu_bar, status_line);
        return Ok(());
    }

    try2_ensure_live_app(worker, line)?;
    let Some(app) = worker.live_turbo_vision_app.as_mut() else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Turbo Vision live session is not initialized",
            "Call `Application.Open` before setting chrome.",
            line,
        ));
    };

    let (terminal_width, terminal_height) = app.terminal.size();

    if let Some(mut menu_bar) = menu_bar {
        layout_menu_bar_for_terminal(&mut menu_bar, terminal_width);
        app.set_menu_bar(menu_bar);
    }

    if let Some(mut status_line) = status_line {
        layout_status_line_for_terminal(&mut status_line, terminal_width, terminal_height);
        app.set_status_line(status_line);
    }

    Ok(())
}

fn build_try2_menu_bar(
    bounds: crate::vm::shared::TurboVisionRect,
    menus: &[TurboVisionMenu],
) -> MenuBar {
    let mut menu_bar = MenuBar::new(turbo_rect(bounds));
    for menu in menus {
        menu_bar.add_submenu(SubMenu::new(&menu.title, build_try2_menu(menu)));
    }
    menu_bar
}

fn build_try2_menu(menu: &TurboVisionMenu) -> Menu {
    Menu::from_items(
        menu.items
            .iter()
            .map(|item| {
                if item.command_id == 0 {
                    MenuItem::separator()
                } else {
                    MenuItem::new(&item.text, item.command_id, 0, 0)
                }
            })
            .collect(),
    )
}

fn build_try2_status_line(
    bounds: crate::vm::shared::TurboVisionRect,
    items: &[TurboVisionStatusItem],
) -> StatusLine {
    StatusLine::new(
        turbo_rect(bounds),
        items
            .iter()
            .map(|item| StatusItem::new(&item.text, item.key_code, item.command_id))
            .collect(),
    )
}

fn try2_session_closed_error(line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        "Try-2 TUI session is not open",
        "Call `Application.Open` before creating Turbo Vision chrome on the try-2 path.",
        line,
    )
}

fn menu_bar_error(error: RegistryError, line: SourceLocation) -> VmError {
    let (message, help) = match error {
        RegistryError::UnknownHandle(handle) => (
            format!("MenuBar handle {handle} is not live"),
            "Use a handle returned by `MenuBar.New`.",
        ),
        RegistryError::WrongKind { handle, .. } => (
            format!("Handle {handle} is not a MenuBar"),
            "Pass a handle from `MenuBar.New`.",
        ),
    };
    runtime_error(RUNTIME_INTRINSIC_STACK_STATE_ERROR, message, help, line)
}

fn status_line_error(error: RegistryError, line: SourceLocation) -> VmError {
    let (message, help) = match error {
        RegistryError::UnknownHandle(handle) => (
            format!("StatusLine handle {handle} is not live"),
            "Use a handle returned by `StatusLine.New`.",
        ),
        RegistryError::WrongKind { handle, .. } => (
            format!("Handle {handle} is not a StatusLine"),
            "Pass a handle from `StatusLine.New`.",
        ),
    };
    runtime_error(RUNTIME_INTRINSIC_STACK_STATE_ERROR, message, help, line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::shared::{TurboVisionMenuItem, TurboVisionRect};
    use turbo_vision::core::command::{CM_ABOUT, CM_OPEN};
    use turbo_vision::core::event::{Event, EventType, KB_F3};
    use turbo_vision::core::menu_data::MenuItem;
    use turbo_vision::views::View;

    fn rect() -> TurboVisionRect {
        TurboVisionRect {
            x: 0,
            y: 0,
            width: 80,
            height: 1,
        }
    }

    #[test]
    fn try2_menu_commands_keep_upstream_ids() {
        let menu = build_try2_menu(&TurboVisionMenu {
            title: "~F~ile".into(),
            items: vec![
                TurboVisionMenuItem {
                    text: "~O~pen".into(),
                    command_id: CM_OPEN,
                },
                TurboVisionMenuItem {
                    text: "-".into(),
                    command_id: 0,
                },
                TurboVisionMenuItem {
                    text: "~A~bout".into(),
                    command_id: CM_ABOUT,
                },
            ],
        });

        assert_eq!(menu.items[0].command(), Some(CM_OPEN));
        assert!(matches!(menu.items[1], MenuItem::Separator));
        assert_eq!(menu.items[2].command(), Some(CM_ABOUT));
    }

    #[test]
    fn try2_status_commands_keep_upstream_ids() {
        let status_line = build_try2_status_line(
            rect(),
            &[TurboVisionStatusItem {
                text: "~F3~ Open".into(),
                key_code: KB_F3,
                command_id: CM_OPEN,
            }],
        );

        let mut event = Event::keyboard(KB_F3);
        let mut status_line = status_line;
        status_line.handle_event(&mut event);
        assert_eq!(event.what, EventType::Command);
        assert_eq!(event.command, CM_OPEN);
    }
}

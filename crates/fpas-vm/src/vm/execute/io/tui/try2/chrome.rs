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
use crate::vm::execute::io::tui::menu_build::build_menu_bar_from_snapshot;
use crate::vm::execute::io::tui::try2::registry::{RegistryError, ViewKind};
use crate::vm::execute::io::tui::try2::session::{Try2MenuBarState, Try2StatusLineState};
use crate::vm::execute::io::tui::tv_geometry::state_rect;
use crate::vm::execute::io::tui::tv_views::{TurboVisionStatusLineSnapshot, build_status_line};
use crate::vm::shared::{TurboVisionMenu, TurboVisionStatusItem};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::geometry::Rect;

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
        .map(|snapshot| build_menu_bar_from_snapshot(snapshot.bounds, &snapshot.menus));
    let status_line = worker.try2.attached_status_line_snapshot().map(|snapshot| {
        build_status_line(TurboVisionStatusLineSnapshot {
            bounds: snapshot.bounds,
            items: snapshot.items.clone(),
        })
    });

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

//! Turbo Vision `Application.Run` integration.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use super::chrome_layout::{layout_menu_bar_for_terminal, layout_status_line_for_terminal};
use super::menu_build::build_menu_bar_from_snapshot;
use super::tv_geometry::turbo_rect;
use super::tv_views::{
    TurboVisionDialogSnapshot, TurboVisionMenuBarSnapshot, TurboVisionStatusLineSnapshot,
    TurboVisionWindowSnapshot, add_dialog_child, add_window_child, build_status_line,
    child_snapshots, radio_groups_from_snapshots,
};
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::TurboVisionObject;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use turbo_vision::app::Application as TurboVisionApplication;
use turbo_vision::views::{dialog::Dialog, window::Window};

const HEADLESS_RUN_MAX_COMMANDS: usize = 4096;

impl Worker {
    pub(in crate::vm::execute::io) fn turbo_vision_application_run(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        if self.current_task_id != 0 {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Application.Run(App) for Turbo Vision must run on the main task",
                "Call `Application.Run(App)` from the main program, not from a `go` task.",
                line,
            ));
        }

        if self.with_tui(|tui| tui.session.is_headless()) {
            self.turbo_vision_begin_run();
            return self.turbo_vision_headless_run(line);
        }

        self.turbo_vision_begin_run();
        self.turbo_vision_refresh_live_desktop(line)?;
        self.turbo_vision_drive_live_interactive_loop(line)
    }

    fn turbo_vision_headless_run(&mut self, line: SourceLocation) -> Result<(), VmError> {
        for _ in 0..HEADLESS_RUN_MAX_COMMANDS {
            let stop = self.with_tui(|tui| {
                tui.turbo_vision.quit_requested
                    || (tui.turbo_vision.pending_commands.is_empty() && !tui.quit_requested)
            });
            if stop {
                return Ok(());
            }
            let _ = self.turbo_vision_pump_next_command(line)?;
            self.turbo_vision_reconcile_after_step(None, line)?;
        }

        Err(runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            format!(
                "Application.Run(App) for Turbo Vision exceeded {HEADLESS_RUN_MAX_COMMANDS} queued command iterations"
            ),
            "Call `Application.Quit(App)` from the command handler or stop queueing commands.",
            line,
        ))
    }

    /// Refresh menu bar and status line on a live Turbo Vision application from FPAS state.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_sync_chrome_from_fpas(
        &self,
        app: &mut TurboVisionApplication,
    ) {
        let (terminal_width, terminal_height) = app.terminal.size();

        if let Some(menu_bar) = self.turbo_vision_menu_bar_snapshot() {
            let mut menu_bar = build_menu_bar_from_snapshot(menu_bar.bounds, &menu_bar.menus);
            layout_menu_bar_for_terminal(&mut menu_bar, terminal_width);
            app.set_menu_bar(menu_bar);
        }

        if let Some(status_line) = self.turbo_vision_status_line_snapshot() {
            let mut status_line = build_status_line(status_line);
            layout_status_line_for_terminal(&mut status_line, terminal_width, terminal_height);
            app.set_status_line(status_line);
        }
    }

    /// Add every on-desktop window and every dialog from current FPAS state to the desktop.
    ///
    /// Shared by the initial build and by the live reconcile rebuild so both paths
    /// construct identical views. Dialogs added this way are not modal and their
    /// input edits are not committed back to FPAS handles; use
    /// `Application.ExecDialog` for modal read-back.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_populate_desktop(
        &self,
        app: &mut TurboVisionApplication,
    ) {
        self.turbo_vision_populate_desktop_on(&mut app.desktop);
    }

    /// Add every on-desktop window and dialog from FPAS state to `desktop`.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_populate_desktop_on(
        &self,
        desktop: &mut turbo_vision::views::desktop::Desktop,
    ) {
        self.turbo_vision_clear_live_view_ids();
        let tree_dirty = self.with_tui(|tui| tui.turbo_vision.pending_reconcile.clone());

        let mut roots = self.with_tui(|tui| {
            let mut roots = Vec::new();
            for (handle, object) in &tui.turbo_vision.objects {
                match object {
                    TurboVisionObject::Window(window) if window.on_desktop => {
                        roots.push(DesktopRootSnapshot {
                            handle: *handle,
                            kind: DesktopRootKind::Window {
                                bounds: window.bounds,
                                title: window.title.clone(),
                                children: window.children.clone(),
                            },
                        });
                    }
                    TurboVisionObject::Dialog(dialog) => {
                        roots.push(DesktopRootSnapshot {
                            handle: *handle,
                            kind: DesktopRootKind::Dialog {
                                bounds: dialog.bounds,
                                title: dialog.title.clone(),
                                children: dialog.children.clone(),
                            },
                        });
                    }
                    _ => {}
                }
            }
            roots.sort_by_key(|root| root.handle);
            roots
        });

        for root in roots {
            match root.kind {
                DesktopRootKind::Window {
                    bounds,
                    title,
                    children: child_handles,
                } => self.turbo_vision_add_window_root_to_desktop(
                    desktop,
                    root.handle,
                    bounds,
                    title,
                    child_handles,
                    tree_dirty.clone(),
                ),
                DesktopRootKind::Dialog {
                    bounds,
                    title,
                    children: child_handles,
                } => self.turbo_vision_add_dialog_root_to_desktop(
                    desktop,
                    root.handle,
                    bounds,
                    title,
                    child_handles,
                    tree_dirty.clone(),
                ),
            }
        }
    }

    fn turbo_vision_add_window_root_to_desktop(
        &self,
        desktop: &mut turbo_vision::views::desktop::Desktop,
        window_handle: u32,
        bounds: crate::vm::shared::TurboVisionRect,
        title: String,
        child_handles: Vec<u32>,
        tree_dirty: crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell,
    ) {
        let children =
            self.with_tui(|tui| child_snapshots(&tui.turbo_vision.objects, &child_handles));
        let radio_groups = radio_groups_from_snapshots(&children);
        let mut window_view = Window::new(turbo_rect(bounds), &title);
        let mut child_registrations = Vec::new();
        for (child_handle, child) in child_handles.into_iter().zip(children) {
            let (view_id, input_line_binding) =
                add_window_child(&mut window_view, child, &radio_groups, tree_dirty.clone());
            if let Some(binding) = input_line_binding {
                self.turbo_vision_register_input_line_view_binding(child_handle, binding);
            }
            child_registrations.push((child_handle, view_id));
        }
        let root_id = desktop.add(Box::new(window_view));
        self.turbo_vision_register_live_view_id(window_handle, root_id);
        for (child_handle, view_id) in child_registrations {
            self.turbo_vision_register_live_child_view(child_handle, root_id, view_id);
        }
    }

    fn turbo_vision_add_dialog_root_to_desktop(
        &self,
        desktop: &mut turbo_vision::views::desktop::Desktop,
        dialog_handle: u32,
        bounds: crate::vm::shared::TurboVisionRect,
        title: String,
        child_handles: Vec<u32>,
        tree_dirty: crate::vm::turbo_vision_bool_cell::TurboVisionBoolCell,
    ) {
        let children =
            self.with_tui(|tui| child_snapshots(&tui.turbo_vision.objects, &child_handles));
        let radio_groups = radio_groups_from_snapshots(&children);
        let mut dialog_view = Dialog::new_modal(turbo_rect(bounds), &title);
        let mut input_bindings = Vec::new();
        let mut child_registrations = Vec::new();
        for (child_handle, child) in child_handles.into_iter().zip(children) {
            let (view_id, input_line_binding) = add_dialog_child(
                &mut dialog_view,
                child,
                child_handle,
                &mut input_bindings,
                &radio_groups,
                tree_dirty.clone(),
            );
            if let Some(binding) = input_line_binding {
                self.turbo_vision_register_input_line_view_binding(child_handle, binding);
            }
            child_registrations.push((child_handle, view_id));
        }
        let root_id = desktop.add(dialog_view);
        self.turbo_vision_register_live_view_id(dialog_handle, root_id);
        for (child_handle, view_id) in child_registrations {
            self.turbo_vision_register_live_child_view(child_handle, root_id, view_id);
        }
    }
}

enum DesktopRootKind {
    Window {
        bounds: crate::vm::shared::TurboVisionRect,
        title: String,
        children: Vec<u32>,
    },
    Dialog {
        bounds: crate::vm::shared::TurboVisionRect,
        title: String,
        children: Vec<u32>,
    },
}

struct DesktopRootSnapshot {
    handle: u32,
    kind: DesktopRootKind,
}

impl Worker {
    pub(in crate::vm::execute::io::tui) fn turbo_vision_window_snapshots(
        &self,
    ) -> Vec<TurboVisionWindowSnapshot> {
        self.with_tui(|tui| {
            let mut snapshots: Vec<_> = tui
                .turbo_vision
                .objects
                .iter()
                .filter_map(|(handle, object)| {
                    let TurboVisionObject::Window(window) = object else {
                        return None;
                    };
                    if !window.on_desktop {
                        return None;
                    }
                    Some((
                        *handle,
                        TurboVisionWindowSnapshot {
                            bounds: window.bounds,
                            title: window.title.clone(),
                            children: child_snapshots(&tui.turbo_vision.objects, &window.children),
                        },
                    ))
                })
                .collect();
            snapshots.sort_by_key(|(handle, _)| *handle);
            snapshots
                .into_iter()
                .map(|(_, snapshot)| snapshot)
                .collect()
        })
    }

    pub(in crate::vm::execute::io::tui) fn turbo_vision_dialog_snapshots(
        &self,
    ) -> Vec<TurboVisionDialogSnapshot> {
        self.with_tui(|tui| {
            let mut snapshots: Vec<_> = tui
                .turbo_vision
                .objects
                .iter()
                .filter_map(|(handle, object)| {
                    let TurboVisionObject::Dialog(dialog) = object else {
                        return None;
                    };
                    Some((
                        *handle,
                        TurboVisionDialogSnapshot {
                            bounds: dialog.bounds,
                            title: dialog.title.clone(),
                            children: child_snapshots(&tui.turbo_vision.objects, &dialog.children),
                        },
                    ))
                })
                .collect();
            snapshots.sort_by_key(|(handle, _)| *handle);
            snapshots
                .into_iter()
                .map(|(_, snapshot)| snapshot)
                .collect()
        })
    }

    pub(in crate::vm::execute::io::tui) fn turbo_vision_menu_bar_snapshot(
        &self,
    ) -> Option<TurboVisionMenuBarSnapshot> {
        self.with_tui(|tui| {
            let handle = tui.turbo_vision.menu_bar?;
            match tui.turbo_vision.objects.get(&handle) {
                Some(TurboVisionObject::MenuBar(menu_bar)) => Some(TurboVisionMenuBarSnapshot {
                    bounds: menu_bar.bounds,
                    menus: menu_bar.menus.clone(),
                }),
                _ => None,
            }
        })
    }

    pub(in crate::vm::execute::io::tui) fn turbo_vision_status_line_snapshot(
        &self,
    ) -> Option<TurboVisionStatusLineSnapshot> {
        self.with_tui(|tui| {
            let handle = tui.turbo_vision.status_line?;
            match tui.turbo_vision.objects.get(&handle) {
                Some(TurboVisionObject::StatusLine(status_line)) => {
                    Some(TurboVisionStatusLineSnapshot {
                        bounds: status_line.bounds,
                        items: status_line.items.clone(),
                    })
                }
                _ => None,
            }
        })
    }
}

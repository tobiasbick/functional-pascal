//! Turbo Vision `Application.Run` integration.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use super::menu_build::build_menu_bar_from_snapshot;
use super::tv_geometry::turbo_rect;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::{
    TurboVisionButton, TurboVisionCheckBox, TurboVisionInputLine, TurboVisionListBox,
    TurboVisionMemo, TurboVisionObject, TurboVisionRadioButton, TurboVisionRect,
    TurboVisionStaticText, TurboVisionStatusItem, TurboVisionTextViewer,
};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use turbo_vision::app::Application as TurboVisionApplication;
use turbo_vision::core::event::{Event, EventType};
use turbo_vision::views::{
    button::Button, checkbox::CheckBox, dialog::Dialog, input_line::InputLine, listbox::ListBox,
    memo::Memo, radiobutton::RadioButton, static_text::StaticText, status_line::StatusItem,
    status_line::StatusLine, text_viewer::TextViewer, window::Window,
};

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
        let mut app = self.build_turbo_vision_application(line)?;
        self.turbo_vision_interactive_run(&mut app, line)
    }

    /// Drive the live Turbo Vision event loop while routing application commands
    /// back into the FPAS `OnCommand` callback.
    ///
    /// Turbo Vision's own `Application::run` consumes every event internally and
    /// silently drops commands it does not recognize, so FPAS code could never
    /// observe button, menu, or status-line actions during an interactive run.
    /// This loop steps the event pump manually: after `handle_event` runs, any
    /// event still typed as a command is one Turbo Vision left unhandled, i.e. an
    /// application command, which is dispatched into the VM. A quit requested from
    /// that callback (`Application.Quit`) or Turbo Vision's own quit (Alt+X) ends
    /// the loop.
    fn turbo_vision_interactive_run(
        &mut self,
        app: &mut TurboVisionApplication,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        app.running = true;
        loop {
            if !app.running
                || self.with_tui(|tui| tui.quit_requested || tui.turbo_vision.quit_requested)
            {
                return Ok(());
            }

            if let Some(mut event) = app.get_event() {
                app.handle_event(&mut event);
                if event.what == EventType::Command {
                    self.dispatch_turbo_vision_command_event(&Event::command(event.command), line)?;
                    self.turbo_vision_reconcile_after_step(Some(app), line)?;
                }
            }

            let _ = app.desktop.remove_closed_windows();
            let _ = app.desktop.handle_moved_windows(&mut app.terminal);
        }
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

    fn build_turbo_vision_application(
        &self,
        line: SourceLocation,
    ) -> Result<TurboVisionApplication, VmError> {
        let window_snapshots = self.turbo_vision_window_snapshots();
        let dialog_snapshots = self.turbo_vision_dialog_snapshots();
        let menu_bar_snapshot = self.turbo_vision_menu_bar_snapshot();
        let status_line_snapshot = self.turbo_vision_status_line_snapshot();
        let mut app = TurboVisionApplication::new().map_err(|error| {
            runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("Turbo Vision terminal initialization failed: {error}"),
                "Run the program from an interactive terminal or use `Application.OpenForTest` in automated tests.",
                line,
            )
        })?;

        if let Some(menu_bar) = menu_bar_snapshot {
            app.set_menu_bar(build_menu_bar_from_snapshot(
                menu_bar.bounds,
                &menu_bar.menus,
            ));
        }

        if let Some(status_line) = status_line_snapshot {
            app.set_status_line(build_status_line(status_line));
        }

        for window in window_snapshots {
            let mut window_view = Window::new(turbo_rect(window.bounds), &window.title);
            for child in window.children {
                add_window_child(&mut window_view, child);
            }
            app.desktop.add(Box::new(window_view));
        }

        for dialog in dialog_snapshots {
            let mut dialog_view = Dialog::new_modal(turbo_rect(dialog.bounds), &dialog.title);
            let mut input_bindings = Vec::new();
            for child in dialog.children {
                add_dialog_child(&mut dialog_view, child, 0, &mut input_bindings);
            }
            app.desktop.add(dialog_view);
        }

        Ok(app)
    }

    fn turbo_vision_window_snapshots(&self) -> Vec<TurboVisionWindowSnapshot> {
        self.with_tui(|tui| {
            tui.turbo_vision
                .objects
                .values()
                .filter_map(|object| {
                    let TurboVisionObject::Window(window) = object else {
                        return None;
                    };
                    if !window.on_desktop {
                        return None;
                    }
                    Some(TurboVisionWindowSnapshot {
                        bounds: window.bounds,
                        title: window.title.clone(),
                        children: child_snapshots(&tui.turbo_vision.objects, &window.children),
                    })
                })
                .collect()
        })
    }

    fn turbo_vision_dialog_snapshots(&self) -> Vec<TurboVisionDialogSnapshot> {
        self.with_tui(|tui| {
            tui.turbo_vision
                .objects
                .values()
                .filter_map(|object| {
                    let TurboVisionObject::Dialog(dialog) = object else {
                        return None;
                    };
                    Some(TurboVisionDialogSnapshot {
                        bounds: dialog.bounds,
                        title: dialog.title.clone(),
                        children: child_snapshots(&tui.turbo_vision.objects, &dialog.children),
                    })
                })
                .collect()
        })
    }

    fn turbo_vision_menu_bar_snapshot(&self) -> Option<TurboVisionMenuBarSnapshot> {
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

    fn turbo_vision_status_line_snapshot(&self) -> Option<TurboVisionStatusLineSnapshot> {
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

struct TurboVisionWindowSnapshot {
    bounds: TurboVisionRect,
    title: String,
    children: Vec<TurboVisionChildSnapshot>,
}

struct TurboVisionDialogSnapshot {
    bounds: TurboVisionRect,
    title: String,
    children: Vec<TurboVisionChildSnapshot>,
}

struct TurboVisionMenuBarSnapshot {
    bounds: TurboVisionRect,
    menus: Vec<crate::vm::shared::TurboVisionMenu>,
}

struct TurboVisionStatusLineSnapshot {
    bounds: TurboVisionRect,
    items: Vec<TurboVisionStatusItem>,
}

pub(in crate::vm::execute::io::tui) enum TurboVisionChildSnapshot {
    Button(TurboVisionButton),
    StaticText(TurboVisionStaticText),
    Memo(TurboVisionMemo),
    TextViewer(TurboVisionTextViewer),
    InputLine(TurboVisionInputLine),
    ListBox(TurboVisionListBox),
    CheckBox(TurboVisionCheckBox),
    RadioButton(TurboVisionRadioButton),
}

pub(in crate::vm::execute::io::tui) fn child_snapshots(
    objects: &std::collections::HashMap<u32, TurboVisionObject>,
    handles: &[u32],
) -> Vec<TurboVisionChildSnapshot> {
    handles
        .iter()
        .filter_map(|handle| match objects.get(handle) {
            Some(TurboVisionObject::Button(button)) => {
                Some(TurboVisionChildSnapshot::Button(button.clone()))
            }
            Some(TurboVisionObject::StaticText(static_text)) => {
                Some(TurboVisionChildSnapshot::StaticText(static_text.clone()))
            }
            Some(TurboVisionObject::Memo(memo)) => {
                Some(TurboVisionChildSnapshot::Memo(memo.clone()))
            }
            Some(TurboVisionObject::TextViewer(text_viewer)) => {
                Some(TurboVisionChildSnapshot::TextViewer(text_viewer.clone()))
            }
            Some(TurboVisionObject::InputLine(input_line)) => {
                Some(TurboVisionChildSnapshot::InputLine(input_line.clone()))
            }
            Some(TurboVisionObject::ListBox(list_box)) => {
                Some(TurboVisionChildSnapshot::ListBox(list_box.clone()))
            }
            Some(TurboVisionObject::CheckBox(check_box)) => {
                Some(TurboVisionChildSnapshot::CheckBox(check_box.clone()))
            }
            Some(TurboVisionObject::RadioButton(radio_button)) => {
                Some(TurboVisionChildSnapshot::RadioButton(radio_button.clone()))
            }
            _ => None,
        })
        .collect()
}

pub(in crate::vm::execute::io::tui) fn add_window_child(
    window: &mut Window,
    child: TurboVisionChildSnapshot,
) {
    match child {
        TurboVisionChildSnapshot::Button(button) => {
            window.add(Box::new(Button::new(
                turbo_rect(button.bounds),
                &button.text,
                button.command_id,
                false,
            )));
        }
        TurboVisionChildSnapshot::StaticText(static_text) => {
            window.add(Box::new(StaticText::new(
                turbo_rect(static_text.bounds),
                &static_text.text,
            )));
        }
        TurboVisionChildSnapshot::Memo(memo) => {
            window.add(Box::new(build_memo(memo)));
        }
        TurboVisionChildSnapshot::TextViewer(text_viewer) => {
            window.add(Box::new(build_text_viewer(text_viewer)));
        }
        TurboVisionChildSnapshot::InputLine(input_line) => {
            window.add(Box::new(InputLine::new(
                turbo_rect(input_line.bounds),
                input_line.max_length,
                input_line.text_cell.view_binding(),
            )));
        }
        TurboVisionChildSnapshot::ListBox(list_box) => {
            window.add(Box::new(build_list_box(list_box)));
        }
        TurboVisionChildSnapshot::CheckBox(check_box) => {
            window.add(Box::new(build_check_box(check_box)));
        }
        TurboVisionChildSnapshot::RadioButton(radio_button) => {
            window.add(Box::new(build_radio_button(radio_button)));
        }
    }
}

fn add_dialog_child(
    dialog: &mut Dialog,
    child: TurboVisionChildSnapshot,
    child_handle: u32,
    input_bindings: &mut Vec<(u32, Rc<RefCell<String>>)>,
) {
    match child {
        TurboVisionChildSnapshot::Button(button) => {
            dialog.add(Box::new(Button::new(
                turbo_rect(button.bounds),
                &button.text,
                button.command_id,
                false,
            )));
        }
        TurboVisionChildSnapshot::StaticText(static_text) => {
            dialog.add(Box::new(StaticText::new(
                turbo_rect(static_text.bounds),
                &static_text.text,
            )));
        }
        TurboVisionChildSnapshot::Memo(memo) => {
            dialog.add(Box::new(build_memo(memo)));
        }
        TurboVisionChildSnapshot::TextViewer(text_viewer) => {
            dialog.add(Box::new(build_text_viewer(text_viewer)));
        }
        TurboVisionChildSnapshot::InputLine(input_line) => {
            let binding = input_line.text_cell.view_binding();
            input_bindings.push((child_handle, binding.clone()));
            dialog.add(Box::new(InputLine::new(
                turbo_rect(input_line.bounds),
                input_line.max_length,
                binding,
            )));
        }
        TurboVisionChildSnapshot::ListBox(list_box) => {
            dialog.add(Box::new(build_list_box(list_box)));
        }
        TurboVisionChildSnapshot::CheckBox(check_box) => {
            dialog.add(Box::new(build_check_box(check_box)));
        }
        TurboVisionChildSnapshot::RadioButton(radio_button) => {
            dialog.add(Box::new(build_radio_button(radio_button)));
        }
    }
}

fn build_list_box(snapshot: TurboVisionListBox) -> ListBox {
    let mut list_box = ListBox::new(turbo_rect(snapshot.bounds), snapshot.command_id);
    list_box.set_items(snapshot.items);
    list_box
}

fn build_check_box(snapshot: TurboVisionCheckBox) -> CheckBox {
    let mut check_box = CheckBox::new(turbo_rect(snapshot.bounds), &snapshot.text);
    check_box.set_checked(snapshot.checked);
    check_box
}

fn build_memo(snapshot: TurboVisionMemo) -> Memo {
    let mut memo = Memo::new(turbo_rect(snapshot.bounds));
    memo.set_text(&snapshot.text);
    memo
}

fn build_text_viewer(snapshot: TurboVisionTextViewer) -> TextViewer {
    let mut text_viewer = TextViewer::new(turbo_rect(snapshot.bounds));
    text_viewer.set_text(&snapshot.text);
    text_viewer
}

fn build_radio_button(snapshot: TurboVisionRadioButton) -> RadioButton {
    let mut radio_button = RadioButton::new(
        turbo_rect(snapshot.bounds),
        &snapshot.text,
        snapshot.group_id,
    );
    if snapshot.selected {
        radio_button.select();
    }
    radio_button
}

fn build_status_line(snapshot: TurboVisionStatusLineSnapshot) -> StatusLine {
    StatusLine::new(
        turbo_rect(snapshot.bounds),
        snapshot
            .items
            .into_iter()
            .map(|item| StatusItem::new(&item.text, item.key_code, item.command_id))
            .collect(),
    )
}

/// Build a modal Turbo Vision dialog view from a live FPAS dialog handle.
pub(in crate::vm::execute::io::tui) fn turbo_vision_build_modal_dialog(
    objects: &HashMap<u32, crate::vm::shared::TurboVisionObject>,
    handle: u32,
    input_bindings: &mut Vec<(u32, Rc<RefCell<String>>)>,
) -> Option<Box<Dialog>> {
    let crate::vm::shared::TurboVisionObject::Dialog(dialog) = objects.get(&handle)? else {
        return None;
    };
    let mut dialog_view = Dialog::new_modal(turbo_rect(dialog.bounds), &dialog.title);
    for child_handle in &dialog.children {
        let Some(child) = child_snapshot(objects, *child_handle) else {
            continue;
        };
        add_dialog_child(&mut dialog_view, child, *child_handle, input_bindings);
    }
    Some(dialog_view)
}

fn child_snapshot(
    objects: &HashMap<u32, crate::vm::shared::TurboVisionObject>,
    handle: u32,
) -> Option<TurboVisionChildSnapshot> {
    match objects.get(&handle) {
        Some(TurboVisionObject::Button(button)) => {
            Some(TurboVisionChildSnapshot::Button(button.clone()))
        }
        Some(TurboVisionObject::StaticText(static_text)) => {
            Some(TurboVisionChildSnapshot::StaticText(static_text.clone()))
        }
        Some(TurboVisionObject::Memo(memo)) => Some(TurboVisionChildSnapshot::Memo(memo.clone())),
        Some(TurboVisionObject::TextViewer(text_viewer)) => {
            Some(TurboVisionChildSnapshot::TextViewer(text_viewer.clone()))
        }
        Some(TurboVisionObject::InputLine(input_line)) => {
            Some(TurboVisionChildSnapshot::InputLine(input_line.clone()))
        }
        Some(TurboVisionObject::ListBox(list_box)) => {
            Some(TurboVisionChildSnapshot::ListBox(list_box.clone()))
        }
        Some(TurboVisionObject::CheckBox(check_box)) => {
            Some(TurboVisionChildSnapshot::CheckBox(check_box.clone()))
        }
        Some(TurboVisionObject::RadioButton(radio_button)) => {
            Some(TurboVisionChildSnapshot::RadioButton(radio_button.clone()))
        }
        _ => None,
    }
}

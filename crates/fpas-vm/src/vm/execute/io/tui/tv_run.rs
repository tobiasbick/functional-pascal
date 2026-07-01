//! Turbo Vision `Application.Run` integration.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use super::tv_geometry::turbo_rect;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::{
    TurboVisionButton, TurboVisionCheckBox, TurboVisionInputLine, TurboVisionListBox,
    TurboVisionObject, TurboVisionRect, TurboVisionStaticText, TurboVisionStatusItem,
};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::app::Application as TurboVisionApplication;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::{
    button::Button, checkbox::CheckBox, dialog::Dialog, input_line::InputLine, listbox::ListBox,
    menu_bar::MenuBar, menu_bar::SubMenu, static_text::StaticText, status_line::StatusItem,
    status_line::StatusLine, window::Window,
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
            return self.turbo_vision_headless_run(line);
        }

        let mut app = self.build_turbo_vision_application(line)?;
        app.run();
        Ok(())
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
            app.set_menu_bar(build_menu_bar(menu_bar));
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
            for child in dialog.children {
                add_dialog_child(&mut dialog_view, child);
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
                    items: menu_bar.items.clone(),
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
    items: Vec<crate::vm::shared::TurboVisionMenuBarItem>,
}

struct TurboVisionStatusLineSnapshot {
    bounds: TurboVisionRect,
    items: Vec<TurboVisionStatusItem>,
}

enum TurboVisionChildSnapshot {
    Button(TurboVisionButton),
    StaticText(TurboVisionStaticText),
    InputLine(TurboVisionInputLine),
    ListBox(TurboVisionListBox),
    CheckBox(TurboVisionCheckBox),
}

fn child_snapshots(
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
            Some(TurboVisionObject::InputLine(input_line)) => {
                Some(TurboVisionChildSnapshot::InputLine(input_line.clone()))
            }
            Some(TurboVisionObject::ListBox(list_box)) => {
                Some(TurboVisionChildSnapshot::ListBox(list_box.clone()))
            }
            Some(TurboVisionObject::CheckBox(check_box)) => {
                Some(TurboVisionChildSnapshot::CheckBox(check_box.clone()))
            }
            _ => None,
        })
        .collect()
}

fn add_window_child(window: &mut Window, child: TurboVisionChildSnapshot) {
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
        TurboVisionChildSnapshot::InputLine(input_line) => {
            window.add(Box::new(InputLine::new(
                turbo_rect(input_line.bounds),
                input_line.max_length,
                Rc::new(RefCell::new(input_line.text)),
            )));
        }
        TurboVisionChildSnapshot::ListBox(list_box) => {
            window.add(Box::new(build_list_box(list_box)));
        }
        TurboVisionChildSnapshot::CheckBox(check_box) => {
            window.add(Box::new(build_check_box(check_box)));
        }
    }
}

fn add_dialog_child(dialog: &mut Dialog, child: TurboVisionChildSnapshot) {
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
        TurboVisionChildSnapshot::InputLine(input_line) => {
            dialog.add(Box::new(InputLine::new(
                turbo_rect(input_line.bounds),
                input_line.max_length,
                Rc::new(RefCell::new(input_line.text)),
            )));
        }
        TurboVisionChildSnapshot::ListBox(list_box) => {
            dialog.add(Box::new(build_list_box(list_box)));
        }
        TurboVisionChildSnapshot::CheckBox(check_box) => {
            dialog.add(Box::new(build_check_box(check_box)));
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

fn build_menu_bar(snapshot: TurboVisionMenuBarSnapshot) -> MenuBar {
    let mut menu_bar = MenuBar::new(turbo_rect(snapshot.bounds));
    for item in snapshot.items {
        let menu = Menu::from_items(vec![MenuItem::new(&item.item_text, item.command_id, 0, 0)]);
        menu_bar.add_submenu(SubMenu::new(&item.menu_text, menu));
    }
    menu_bar
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

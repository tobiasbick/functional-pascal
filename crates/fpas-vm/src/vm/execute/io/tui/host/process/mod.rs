//! Hosted `Std.Tui` event polling and dispatch orchestration.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (from the repository root).

mod callbacks;
mod command;
mod focus;
mod key;
mod pointer;

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::SourceLocation;
use fpas_std::{UiEvent, UiResize, ViewId};

#[derive(Clone, Copy)]
struct DispatchTags {
    hit: i64,
    miss: i64,
}

impl Worker {
    /// Processes at most one pending `UiEvent`, dispatching to the registered handler.
    ///
    /// Returns a status tag: `0` = none, `1` = key dispatched, `2` = resize dispatched,
    /// `3` = key (no handler), `4` = resize (no handler), `5`/`7`/`8`/`9`/`10`/`11`/`12`/`13`
    /// for mouse/paste/focus events (dispatched or not), `18` = key blocked by active modal
    /// scope, `19` = mouse blocked by active modal scope, `20` = command blocked by active
    /// modal scope, `22` = key handler returned `false` (not consumed).
    pub(in crate::vm::execute::io) fn tui_host_process_next_inner(
        &mut self,
        max_spins: usize,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        let mut ready: Option<UiEvent> = None;
        for _ in 0..max_spins.max(1) {
            let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(event) = tui.host.pop_ready_event() {
                ready = Some(event);
                break;
            }
            let polled = self.with_console_and_key_input(|console, key_input| {
                tui.session.poll_ui_event_all(console, key_input, line)
            })?;
            match polled {
                None => break,
                Some(event) => {
                    tui.host.ingest_ui_event(event);
                    if let Some(event) = tui.host.pop_ready_event() {
                        ready = Some(event);
                        break;
                    }
                }
            }
        }

        let Some(event) = ready else {
            return Ok(0);
        };

        let (on_key, on_mouse, on_paste, on_focus_gained, on_focus_lost, on_resize) = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            (
                tui.on_key_pressed.clone(),
                tui.on_mouse.clone(),
                tui.on_paste.clone(),
                tui.on_focus_gained.clone(),
                tui.on_focus_lost.clone(),
                tui.on_resize.clone(),
            )
        };

        let app_rec = Self::tui_application_record();
        let modal_scope = self.active_modal_scope();

        match event {
            UiEvent::Key(key_event) => self.dispatch_tui_key_event(
                key_event,
                on_key,
                app_rec,
                modal_scope.as_deref(),
                line,
            ),
            UiEvent::Mouse(mouse) => self.dispatch_tui_mouse_event(
                mouse,
                on_mouse,
                app_rec,
                modal_scope.as_deref(),
                line,
            ),
            UiEvent::Paste(text) => self.dispatch_console_event_handler(
                on_paste,
                [app_rec, Self::console_paste_event_record(text)],
                Some(self.focused_view_redraw_hint()),
                DispatchTags { hit: 8, miss: 9 },
                line,
            ),
            UiEvent::FocusGained => self.dispatch_console_event_handler(
                on_focus_gained,
                [app_rec, Self::console_focus_gained_event_record()],
                Some(self.focused_view_redraw_hint()),
                DispatchTags { hit: 10, miss: 11 },
                line,
            ),
            UiEvent::FocusLost => self.dispatch_console_event_handler(
                on_focus_lost,
                [app_rec, Self::console_focus_lost_event_record()],
                Some(self.focused_view_redraw_hint()),
                DispatchTags { hit: 12, miss: 13 },
                line,
            ),
            UiEvent::Resize(UiResize {
                old_width,
                old_height,
                width,
                height,
            }) => {
                let old_width = old_width.unwrap_or(width);
                let old_height = old_height.unwrap_or(height);
                self.with_tui(|tui| {
                    tui.session
                        .request_resize_redraw(old_width, old_height, width, height, line)
                })?;
                if let Some(handler) = on_resize {
                    let _ = self.call_function_sync_allowing_shutdown(
                        &handler,
                        &[app_rec, Self::tui_size_record(width, height)],
                        line,
                    )?;
                    Ok(2)
                } else {
                    Ok(4)
                }
            }
            UiEvent::CloseRequested | UiEvent::Wheel(_) => Ok(0),
        }
    }

    fn active_modal_scope(&self) -> Option<Vec<ViewId>> {
        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let scope = Self::modal_scope_ids(&tui);
        (!scope.is_empty()).then_some(scope)
    }
}

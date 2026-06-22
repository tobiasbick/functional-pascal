//! Keyboard focus traversal, widget routing, commands, and fallback callbacks.

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;
use fpas_std::{BlockedInput, ConsoleKeyEvent, FocusDirection, ProcessOutcome, ViewId};

/// Discriminant of `Std.Console.KeyKind.Tab`; must match
/// [`fpas_std::key_event::KEY_KIND_VARIANTS`] (index 2).
const KEY_KIND_TAB: usize = 2;

impl Worker {
    /// Routes one key event through focus traversal, widgets, commands, and Pascal fallback.
    pub(super) fn dispatch_tui_key_event(
        &mut self,
        key_event: ConsoleKeyEvent,
        on_key: Option<Value>,
        app_rec: Value,
        modal_scope: Option<&[ViewId]>,
        line: SourceLocation,
    ) -> Result<ProcessOutcome, VmError> {
        if key_event.kind == KEY_KIND_TAB {
            let (changed, had_previous, previous_focus, current_focus) = {
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                let previous_focus = tui.views.focused_id();
                if let Some(scope) = modal_scope {
                    let (changed, had_previous) = if key_event.shift {
                        tui.views.focus_prev_in_scope(scope)
                    } else {
                        tui.views.focus_next_in_scope(scope)
                    };
                    (
                        changed,
                        had_previous,
                        previous_focus,
                        tui.views.focused_id(),
                    )
                } else if key_event.shift {
                    let (changed, had_previous) = tui.views.focus_prev();
                    (
                        changed,
                        had_previous,
                        previous_focus,
                        tui.views.focused_id(),
                    )
                } else {
                    let (changed, had_previous) = tui.views.focus_next();
                    (
                        changed,
                        had_previous,
                        previous_focus,
                        tui.views.focused_id(),
                    )
                }
            };
            if changed {
                self.request_focus_transition_redraw(previous_focus, current_focus, line)?;
                self.invoke_focus_transition(had_previous, line)?;
                return Ok(ProcessOutcome::FocusMoved(if key_event.shift {
                    FocusDirection::Backward
                } else {
                    FocusDirection::Forward
                }));
            }
        }
        if let Some(tag) = self.try_dispatch_control_key(&key_event, line)? {
            return Ok(tag);
        }
        if let Some(tag) = self.try_dispatch_widget_key(key_event.clone(), modal_scope, line)? {
            return Ok(tag);
        }
        if let Some(tag) = self.try_dispatch_frame_key(&key_event, line)? {
            return Ok(tag);
        }
        if let Some(command) = self.resolve_tui_modal_command(&key_event) {
            return self.dispatch_tui_command(command, line);
        }
        if let Some(command) = self.resolve_tui_scoped_command(&key_event) {
            if self.modal_blocks_keyboard_dispatch(modal_scope) {
                return Ok(ProcessOutcome::Blocked(BlockedInput::Command));
            }
            return self.dispatch_tui_command(command, line);
        }
        if self.modal_blocks_keyboard_dispatch(modal_scope) {
            return Ok(ProcessOutcome::Blocked(BlockedInput::Key));
        }
        if let Some(handler) = on_key {
            let consumed = self.call_function_sync_allowing_shutdown(
                &handler,
                &[app_rec, Self::key_event_record(key_event)],
                line,
            )?;
            match consumed {
                Value::Boolean(consumed) => Ok(ProcessOutcome::Key {
                    handled: true,
                    consumed,
                }),
                other => Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "OnKeyPressed must return boolean, got {}",
                        other.type_name()
                    ),
                    "Return `true` when the application consumed the key or `false` otherwise.",
                    line,
                )),
            }
        } else {
            Ok(ProcessOutcome::Key {
                handled: false,
                consumed: false,
            })
        }
    }

    fn modal_blocks_keyboard_dispatch(&self, modal_scope: Option<&[ViewId]>) -> bool {
        let Some(scope) = modal_scope else {
            return false;
        };

        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        match tui.views.focused_id() {
            Some(focused) => !scope.contains(&focused),
            None => false,
        }
    }
}

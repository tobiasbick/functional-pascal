//! Hosted `Std.Tui` event processing and command dispatch.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;
use fpas_std::{CommandId, DamageRegion, UiEvent, UiMouse, UiResize, ViewId};

/// Discriminant of `Std.Console.KeyKind.Tab`; must match
/// [`fpas_std::key_event::KEY_KIND_VARIANTS`] (index 2).
const KEY_KIND_TAB: usize = 2;

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
            UiEvent::Key(key_event) => {
                if key_event.kind == KEY_KIND_TAB {
                    let (changed, had_previous, previous_focus, current_focus) = {
                        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                        let previous_focus = tui.views.focused_id();
                        if let Some(scope) = modal_scope.as_deref() {
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
                        return Ok(if key_event.shift { 15 } else { 14 });
                    }
                }
                if let Some(tag) =
                    self.try_dispatch_widget_key(key_event.clone(), modal_scope.as_deref(), line)?
                {
                    return Ok(tag);
                }
                if let Some(command_id) = self.resolve_tui_command(&key_event) {
                    if self.modal_blocks_keyboard_dispatch(modal_scope.as_deref()) {
                        return Ok(20);
                    }
                    return self.dispatch_tui_command(command_id, line);
                }
                if self.modal_blocks_keyboard_dispatch(modal_scope.as_deref()) {
                    return Ok(18);
                }
                if let Some(handler) = on_key {
                    let consumed = self.call_function_sync_allowing_shutdown(
                        &handler,
                        &[app_rec, Self::key_event_record(key_event)],
                        line,
                    )?;
                    match consumed {
                        Value::Boolean(true) => Ok(1),
                        Value::Boolean(false) => Ok(22),
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
                    Ok(3)
                }
            }
            UiEvent::Mouse(UiMouse {
                action,
                button,
                x,
                y,
                modifiers,
            }) => {
                let mouse = UiMouse {
                    action,
                    button,
                    x,
                    y,
                    modifiers,
                };
                if self.modal_blocks_mouse_dispatch(modal_scope.as_deref(), mouse) {
                    return Ok(19);
                }
                if let Some(tag) =
                    self.try_dispatch_widget_mouse(mouse, modal_scope.as_deref(), line)?
                {
                    return Ok(tag);
                }
                let redraw_hint = self.mouse_redraw_hint(modal_scope.as_deref(), mouse);
                self.dispatch_console_event_handler(
                    on_mouse,
                    [app_rec, Self::console_mouse_event_record(mouse)],
                    Some(redraw_hint),
                    DispatchTags { hit: 5, miss: 7 },
                    line,
                )
            }
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

    /// Dispatches a `Std.Console.Event`-bearing handler.
    ///
    /// Returns `hit_tag` when `handler` is present (and calls it), `miss_tag` otherwise.
    fn dispatch_console_event_handler(
        &mut self,
        handler: Option<Value>,
        args: [Value; 2],
        redraw_hint: Option<DamageRegion>,
        tags: DispatchTags,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        if let Some(handler) = handler {
            let _ = self.call_function_sync_allowing_shutdown_with_redraw_hint(
                &handler,
                &args,
                redraw_hint,
                line,
            )?;
            Ok(tags.hit)
        } else {
            Ok(tags.miss)
        }
    }

    fn resolve_tui_command(&self, key: &fpas_std::ConsoleKeyEvent) -> Option<CommandId> {
        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(focused) = tui.views.focused_id() {
            for view_id in tui.views.ancestors_inclusive(focused) {
                if let Some(commands) = tui.view_commands.get(&view_id)
                    && let Some(command_id) = commands.resolve(key)
                {
                    return Some(command_id);
                }
            }
        }

        if let Some(command_id) = tui.modals.resolve_active_command(key) {
            return Some(command_id);
        }

        tui.commands.resolve(key)
    }

    pub(in crate::vm::execute::io) fn dispatch_tui_command(
        &mut self,
        command_id: CommandId,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        let handler = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.on_command.clone()
        };
        let Some(handler) = handler else {
            return Ok(17);
        };

        let app_rec = Self::tui_application_record();
        let _ = self.call_function_sync_allowing_shutdown(
            &handler,
            &[app_rec, Value::Integer(command_id.0)],
            line,
        )?;
        Ok(16)
    }

    fn active_modal_scope(&self) -> Option<Vec<ViewId>> {
        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let scope = Self::modal_scope_ids(&tui);
        (!scope.is_empty()).then_some(scope)
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

    fn modal_blocks_mouse_dispatch(&self, modal_scope: Option<&[ViewId]>, mouse: UiMouse) -> bool {
        let Some(scope) = modal_scope else {
            return false;
        };

        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        !scope.iter().any(|view_id| {
            tui.views
                .rect(*view_id)
                .is_some_and(|rect| rect.contains_console_mouse(mouse.x, mouse.y))
        })
    }

    fn request_focus_transition_redraw(
        &self,
        previous_focus: Option<ViewId>,
        current_focus: Option<ViewId>,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let previous_rect = previous_focus.and_then(|view_id| tui.views.rect(view_id));
        let current_rect = current_focus.and_then(|view_id| tui.views.rect(view_id));

        let mut marked_any = false;
        if let Some(rect) = previous_rect {
            tui.session.request_redraw_rect(rect, line)?;
            marked_any = true;
        }
        if let Some(rect) = current_rect {
            tui.session.request_redraw_rect(rect, line)?;
            marked_any = true;
        }
        if !marked_any {
            tui.session.request_redraw(line)?;
        }

        Ok(())
    }

    fn call_function_sync_allowing_shutdown_with_redraw_hint(
        &mut self,
        handler: &Value,
        args: &[Value],
        redraw_hint: Option<DamageRegion>,
        line: SourceLocation,
    ) -> Result<Value, VmError> {
        {
            let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(damage) = redraw_hint {
                tui.session.set_host_redraw_hint(damage);
            } else {
                tui.session.clear_host_redraw_hint();
            }
        }

        let result = self.call_function_sync_allowing_shutdown(handler, args, line);

        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        tui.session.clear_host_redraw_hint();

        result
    }

    fn focused_view_redraw_hint(&self) -> DamageRegion {
        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        tui.views
            .focused_id()
            .and_then(|view_id| tui.views.rect(view_id))
            .map(DamageRegion::Rect)
            .unwrap_or(DamageRegion::FullFrame)
    }

    fn mouse_redraw_hint(&self, modal_scope: Option<&[ViewId]>, mouse: UiMouse) -> DamageRegion {
        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        tui.views
            .topmost_view_at(
                mouse.x.saturating_sub(1),
                mouse.y.saturating_sub(1),
                modal_scope,
            )
            .and_then(|view_id| tui.views.rect(view_id))
            .map(DamageRegion::Rect)
            .unwrap_or(DamageRegion::FullFrame)
    }

    /// Fires `OnDeactivate` (if `fire_deactivate` is `true` and a handler is registered)
    /// then `OnActivate` (if registered) after a focus transition.
    ///
    /// Both handlers have the signature `procedure (Application)`.
    fn invoke_focus_transition(
        &mut self,
        fire_deactivate: bool,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let app_rec = Self::tui_application_record();

        if fire_deactivate {
            let handler = {
                let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.on_deactivate.clone()
            };
            if let Some(handler) = handler {
                let _ = self.call_function_sync_allowing_shutdown(
                    &handler,
                    std::slice::from_ref(&app_rec),
                    line,
                )?;
            }
        }

        let handler = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.on_activate.clone()
        };
        if let Some(handler) = handler {
            let _ = self.call_function_sync_allowing_shutdown(&handler, &[app_rec], line)?;
        }

        Ok(())
    }
}

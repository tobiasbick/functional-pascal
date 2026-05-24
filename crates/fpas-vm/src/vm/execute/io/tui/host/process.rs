//! Hosted `Std.Tui` event processing and command dispatch.
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{
    CommandId, ConsoleEvent, DamageRegion, UiEvent, UiMouse, UiResize, ViewId, ViewRect,
};

/// Discriminant of `Std.Console.KeyKind.Tab`; must match
/// [`fpas_std::key_event::KEY_KIND_VARIANTS`] (index 2).
const KEY_KIND_TAB: usize = 2;

impl Worker {
    /// Processes at most one pending `UiEvent`, dispatching to the registered handler.
    ///
    /// Returns a status tag: `0` = none, `1` = key dispatched, `2` = resize dispatched,
    /// `3` = key (no handler), `4` = resize (no handler), `5`/`7`/`8`/`9`/`10`/`11`/`12`/`13`
    /// for mouse/paste/focus events (dispatched or not), `18` = key blocked by active modal
    /// scope, `19` = mouse blocked by active modal scope, `20` = command blocked by active
    /// modal scope.
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
                tui.session.poll_event_all(console, key_input, line)
            })?;
            match polled {
                None => break,
                Some(tui_event) => {
                    tui.host.ingest_tui_event(tui_event);
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
                    let _ = self.call_function_sync_allowing_shutdown(
                        &handler,
                        &[app_rec, Self::key_event_record(key_event)],
                        line,
                    )?;
                    Ok(1)
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
                let console_event = ConsoleEvent::mouse(
                    action,
                    button,
                    x,
                    y,
                    modifiers.shift,
                    modifiers.ctrl,
                    modifiers.alt,
                    modifiers.meta,
                );
                if self.modal_blocks_mouse_dispatch(modal_scope.as_deref(), &console_event) {
                    return Ok(19);
                }
                let redraw_hint = self.mouse_redraw_hint(modal_scope.as_deref(), &console_event);
                self.dispatch_console_event_handler(
                    on_mouse,
                    app_rec,
                    Self::console_event_record(console_event),
                    Some(redraw_hint),
                    5,
                    7,
                    line,
                )
            }
            UiEvent::Paste(console_event) => self.dispatch_console_event_handler(
                on_paste,
                app_rec,
                Self::console_event_record(console_event),
                Some(self.focused_view_redraw_hint()),
                8,
                9,
                line,
            ),
            UiEvent::FocusGained(console_event) => self.dispatch_console_event_handler(
                on_focus_gained,
                app_rec,
                Self::console_event_record(console_event),
                Some(self.focused_view_redraw_hint()),
                10,
                11,
                line,
            ),
            UiEvent::FocusLost(console_event) => self.dispatch_console_event_handler(
                on_focus_lost,
                app_rec,
                Self::console_event_record(console_event),
                Some(self.focused_view_redraw_hint()),
                12,
                13,
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
        app_rec: Value,
        event_rec: Value,
        redraw_hint: Option<DamageRegion>,
        hit_tag: i64,
        miss_tag: i64,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        if let Some(handler) = handler {
            let _ = self.call_function_sync_allowing_shutdown_with_redraw_hint(
                &handler,
                &[app_rec, event_rec],
                redraw_hint,
                line,
            )?;
            Ok(hit_tag)
        } else {
            Ok(miss_tag)
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

    fn dispatch_tui_command(
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

    fn modal_blocks_mouse_dispatch(
        &self,
        modal_scope: Option<&[ViewId]>,
        event: &ConsoleEvent,
    ) -> bool {
        let Some(scope) = modal_scope else {
            return false;
        };

        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        !scope.iter().any(|view_id| {
            tui.views
                .rect(*view_id)
                .is_some_and(|rect| Self::rect_contains_point(rect, event.mouse_x, event.mouse_y))
        })
    }

    fn rect_contains_point(rect: ViewRect, x: i64, y: i64) -> bool {
        x >= rect.x && y >= rect.y && x < rect.x + rect.width && y < rect.y + rect.height
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

    fn mouse_redraw_hint(
        &self,
        modal_scope: Option<&[ViewId]>,
        event: &ConsoleEvent,
    ) -> DamageRegion {
        let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        tui.views
            .topmost_view_at(event.mouse_x, event.mouse_y, modal_scope)
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
                let _ =
                    self.call_function_sync_allowing_shutdown(&handler, &[app_rec.clone()], line)?;
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

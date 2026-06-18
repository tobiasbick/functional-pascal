//! Hosted `Std.Graph` event processing and handler dispatch.
//!
//! **Documentation:** `docs/pascal/std/graph/app.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::SourceLocation;
use fpas_std::{GraphEvent, UiEvent, UiMouse, UiResize, UiWheel};

impl Worker {
    /// Processes at most one pending [`UiEvent`], dispatching to registered handlers.
    pub(in crate::vm::execute::io) fn graph_host_process_next_inner(
        &mut self,
        max_spins: usize,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        let mut ready: Option<UiEvent> = None;
        for _ in 0..max_spins.max(1) {
            let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(event) = graph.host.pop_ready_event() {
                ready = Some(event);
                break;
            }
            let polled = graph.session.read_host_ui_event_timeout(0, line)?;
            match polled {
                None => break,
                Some(event) => {
                    graph.host.ingest_ui_event(event);
                    if let Some(event) = graph.host.pop_ready_event() {
                        ready = Some(event);
                        break;
                    }
                }
            }
        }

        let Some(event) = ready else {
            return Ok(0);
        };

        let (on_key, on_mouse, on_wheel, on_resize, on_close_requested) = self.with_graph(|g| {
            (
                g.on_key_pressed.clone(),
                g.on_mouse.clone(),
                g.on_wheel.clone(),
                g.on_resize.clone(),
                g.on_close_requested.clone(),
            )
        });

        let app_rec = Self::graph_application_record();

        match event {
            UiEvent::Key(key_event) => {
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
                let graph_event = GraphEvent::Mouse {
                    action,
                    button,
                    x,
                    y,
                    shift: modifiers.shift,
                    ctrl: modifiers.ctrl,
                    alt: modifiers.alt,
                    meta: modifiers.meta,
                };
                if let Some(handler) = on_mouse {
                    let _ = self.call_function_sync_allowing_shutdown(
                        &handler,
                        &[app_rec, Self::graph_event_record(graph_event)],
                        line,
                    )?;
                    Ok(5)
                } else {
                    Ok(7)
                }
            }
            UiEvent::Wheel(UiWheel {
                delta_x,
                delta_y,
                x,
                y,
                modifiers,
            }) => {
                let graph_event = GraphEvent::Wheel {
                    delta_x,
                    delta_y,
                    x,
                    y,
                    shift: modifiers.shift,
                    ctrl: modifiers.ctrl,
                    alt: modifiers.alt,
                    meta: modifiers.meta,
                };
                if let Some(handler) = on_wheel {
                    let _ = self.call_function_sync_allowing_shutdown(
                        &handler,
                        &[app_rec, Self::graph_event_record(graph_event)],
                        line,
                    )?;
                    Ok(8)
                } else {
                    Ok(9)
                }
            }
            UiEvent::Resize(UiResize { width, height, .. }) => {
                self.with_graph(|graph| graph.session.request_redraw(line))?;
                if let Some(handler) = on_resize {
                    let _ = self.call_function_sync_allowing_shutdown(
                        &handler,
                        &[app_rec, Self::graph_size_record(width, height)],
                        line,
                    )?;
                    Ok(2)
                } else {
                    Ok(4)
                }
            }
            UiEvent::CloseRequested => {
                if let Some(handler) = on_close_requested {
                    let _ = self.call_function_sync_allowing_shutdown(
                        &handler,
                        std::slice::from_ref(&app_rec),
                        line,
                    )?;
                }
                self.with_graph(|graph| graph.window_closed = true);
                Ok(10)
            }
            UiEvent::Paste(_) | UiEvent::FocusGained | UiEvent::FocusLost => Ok(0),
        }
    }
}

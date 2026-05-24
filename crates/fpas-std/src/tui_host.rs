//! Rust-hosted TUI event normalization and coalescing (framework plan Phase 2).
//!
//! This module does **not** call FP bytecode; it prepares a single place for the
//! future VM bridge to consume normalized terminal input.
//!
//! - Dispatch-mode behavior spec: `docs/pascal/std/tui-app.md` (from the repository root)
//! - Plan: `docs/future/tui-application-framework.md` (from the repository root)

use crate::console::{Console, KeyInput};
use crate::error::StdError;
use crate::tui::{TuiEvent, TuiSession};
use crate::{UiEvent, UiResize};
use fpas_bytecode::SourceLocation;
use std::collections::VecDeque;

type TraceFn = fn(&str);

/// Coalesces rapid [`TuiEvent::Resize`] bursts and exposes a small ready queue
/// so a key following resizes yields **`Resize` (once, last size) then `Key`**.
#[derive(Debug, Default)]
pub struct TuiHost {
    pending_resize: Option<(i64, i64, i64, i64)>,
    ready: VecDeque<UiEvent>,
    trace: Option<TraceFn>,
}

impl TuiHost {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Optional structured trace (no dependency on `log` / `tracing`).
    pub fn set_trace_hook(&mut self, hook: Option<TraceFn>) {
        self.trace = hook;
    }

    fn trace(&self, msg: &'static str) {
        if let Some(f) = self.trace {
            f(msg);
        }
    }

    /// Feed one mapped [`TuiEvent`] from the session / console path.
    pub fn ingest_tui_event(&mut self, ev: TuiEvent) {
        match ev.into_ui_event() {
            UiEvent::Resize(UiResize {
                old_width: Some(old_width),
                old_height: Some(old_height),
                width,
                height,
            }) => {
                self.trace("tui_host: buffer resize (coalesce)");
                self.pending_resize = Some(
                    self.pending_resize
                        .map_or((old_width, old_height, width, height), |pending| {
                            (pending.0, pending.1, width, height)
                        }),
                );
            }
            UiEvent::Key(key) => {
                if let Some((old_width, old_height, width, height)) = self.pending_resize.take() {
                    self.trace("tui_host: flush coalesced resize before key");
                    self.ready.push_back(UiEvent::Resize(UiResize::new(
                        Some(old_width),
                        Some(old_height),
                        width,
                        height,
                    )));
                }
                self.ready.push_back(UiEvent::Key(key));
            }
            UiEvent::Mouse(mouse) => {
                if let Some((old_width, old_height, width, height)) = self.pending_resize.take() {
                    self.trace("tui_host: flush coalesced resize before mouse");
                    self.ready.push_back(UiEvent::Resize(UiResize::new(
                        Some(old_width),
                        Some(old_height),
                        width,
                        height,
                    )));
                }
                self.ready.push_back(UiEvent::Mouse(mouse));
            }
            UiEvent::Paste(text) => {
                if let Some((old_width, old_height, width, height)) = self.pending_resize.take() {
                    self.trace("tui_host: flush coalesced resize before paste");
                    self.ready.push_back(UiEvent::Resize(UiResize::new(
                        Some(old_width),
                        Some(old_height),
                        width,
                        height,
                    )));
                }
                self.ready.push_back(UiEvent::Paste(text));
            }
            UiEvent::FocusGained => {
                if let Some((old_width, old_height, width, height)) = self.pending_resize.take() {
                    self.trace("tui_host: flush coalesced resize before focus-gained");
                    self.ready.push_back(UiEvent::Resize(UiResize::new(
                        Some(old_width),
                        Some(old_height),
                        width,
                        height,
                    )));
                }
                self.ready.push_back(UiEvent::FocusGained);
            }
            UiEvent::FocusLost => {
                if let Some((old_width, old_height, width, height)) = self.pending_resize.take() {
                    self.trace("tui_host: flush coalesced resize before focus-lost");
                    self.ready.push_back(UiEvent::Resize(UiResize::new(
                        Some(old_width),
                        Some(old_height),
                        width,
                        height,
                    )));
                }
                self.ready.push_back(UiEvent::FocusLost);
            }
            UiEvent::CloseRequested | UiEvent::Wheel(_) | UiEvent::Resize(_) => {}
        }
    }

    /// Emit a single coalesced [`UiEvent::Resize`] if a resize was buffered and no key arrived yet.
    ///
    /// Intended for an **idle** or **timeout** tick in the outer host loop (see plan Phase 2).
    #[must_use]
    pub fn flush_pending_resize(&mut self) -> bool {
        if let Some((old_width, old_height, width, height)) = self.pending_resize.take() {
            self.trace("tui_host: flush pending resize (idle)");
            self.ready.push_back(UiEvent::Resize(UiResize::new(
                Some(old_width),
                Some(old_height),
                width,
                height,
            )));
            return true;
        }
        false
    }

    #[must_use]
    pub fn pop_ready_event(&mut self) -> Option<UiEvent> {
        self.ready.pop_front()
    }

    #[must_use]
    pub fn peek_ready_event(&self) -> Option<&UiEvent> {
        self.ready.front()
    }

    /// Non-blocking: returns a ready [`UiEvent`] or polls the session once.
    pub fn poll_next(
        &mut self,
        session: &TuiSession,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<Option<UiEvent>, StdError> {
        if let Some(ev) = self.pop_ready_event() {
            return Ok(Some(ev));
        }

        match session.poll_event(console, key_input, location)? {
            None => Ok(None),
            Some(tui) => {
                self.ingest_tui_event(tui);
                Ok(self.pop_ready_event())
            }
        }
    }

    /// Blocking: read until at least one [`UiEvent`] is returned (resize-only bursts keep reading).
    pub fn read_next_blocking(
        &mut self,
        session: &TuiSession,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<UiEvent, StdError> {
        loop {
            if let Some(ev) = self.pop_ready_event() {
                return Ok(ev);
            }

            let tui = session.read_event(console, key_input, location)?;
            self.ingest_tui_event(tui);
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;
    use crate::ConsoleKeyEvent;
    use crate::console_event::ConsoleEvent;
    use crate::key_event::key_kind_index;

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn coalesces_multiple_resizes_before_key() {
        let mut host = TuiHost::new();
        host.ingest_tui_event(TuiEvent::Resize {
            old_width: 80,
            old_height: 25,
            width: 10,
            height: 10,
        });
        host.ingest_tui_event(TuiEvent::Resize {
            old_width: 10,
            old_height: 10,
            width: 20,
            height: 30,
        });
        host.ingest_tui_event(TuiEvent::Key(ConsoleKeyEvent::new(
            key_kind_index("Space"),
            ' ',
            false,
            false,
            false,
            false,
        )));

        assert_eq!(
            host.pop_ready_event(),
            Some(UiEvent::Resize(UiResize::new(Some(80), Some(25), 20, 30,)))
        );
        assert!(matches!(host.pop_ready_event(), Some(UiEvent::Key(_))));
        assert_eq!(host.pop_ready_event(), None);
    }

    #[test]
    fn flush_pending_resize_after_resize_only_stream() {
        let mut host = TuiHost::new();
        host.ingest_tui_event(TuiEvent::Resize {
            old_width: 80,
            old_height: 25,
            width: 80,
            height: 25,
        });
        assert!(host.flush_pending_resize());
        assert_eq!(
            host.pop_ready_event(),
            Some(UiEvent::Resize(UiResize::new(Some(80), Some(25), 80, 25,)))
        );
        assert!(!host.flush_pending_resize());
    }

    #[test]
    fn read_next_blocking_drains_resize_burst_then_key() {
        let mut session = TuiSession::default();
        let mut console = Console::new();
        let mut key_input = KeyInput::new();
        let mut host = TuiHost::new();

        session
            .open(&mut console, &mut key_input, loc())
            .expect("open");
        key_input.push_console_event(ConsoleEvent::resize(1, 1));
        key_input.push_console_event(ConsoleEvent::resize(100, 40));
        key_input.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
            key_kind_index("Escape"),
            '\u{1b}',
            false,
            false,
            false,
            false,
        )));

        let first = host
            .read_next_blocking(&session, &mut console, &mut key_input, loc())
            .expect("resize");
        assert_eq!(
            first,
            UiEvent::Resize(UiResize::new(Some(80), Some(25), 100, 40))
        );
        let second = host
            .read_next_blocking(&session, &mut console, &mut key_input, loc())
            .expect("key");
        assert!(matches!(second, UiEvent::Key(_)));
    }

    #[test]
    fn trace_hook_runs_on_resize_and_flush() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALLS: AtomicUsize = AtomicUsize::new(0);

        fn hook(_: &str) {
            CALLS.fetch_add(1, Ordering::Relaxed);
        }

        CALLS.store(0, Ordering::Relaxed);
        let mut host = TuiHost::new();
        host.set_trace_hook(Some(hook as TraceFn));
        host.ingest_tui_event(TuiEvent::Resize {
            old_width: 80,
            old_height: 25,
            width: 1,
            height: 1,
        });
        assert!(CALLS.load(Ordering::Relaxed) >= 1);
        let before = CALLS.load(Ordering::Relaxed);
        assert!(host.flush_pending_resize());
        assert!(CALLS.load(Ordering::Relaxed) > before);
    }

    #[test]
    fn resize_suggests_redraw_key_does_not() {
        let r = UiEvent::Resize(UiResize::new(Some(80), Some(25), 1, 1));
        assert!(r.suggests_request_redraw());
        let k = UiEvent::Key(crate::ConsoleKeyEvent::new(
            key_kind_index("A"),
            'a',
            false,
            false,
            false,
            false,
        ));
        assert!(!k.suggests_request_redraw());
    }
}

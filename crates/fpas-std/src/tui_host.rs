//! Rust-hosted TUI event normalization and coalescing (framework plan Phase 2).
//!
//! This module does **not** call FP bytecode; it prepares a single place for the
//! future VM bridge to consume normalized terminal input.
//!
//! - Dispatch-mode behavior spec: `docs/pascal/std/tui-app.md` (from the repository root)
//! - Plan: `docs/future/tui-application-framework.md` (from the repository root)

use crate::ConsoleKeyEvent;
use crate::console::{Console, KeyInput};
use crate::error::StdError;
use crate::tui::{TuiEvent, TuiSession};
use fpas_bytecode::SourceLocation;
use std::collections::VecDeque;

/// Normalized input for a future hosted main loop (`Application.Run`).
///
/// Maps 1:1 from [`TuiEvent`] today; extra variants (paste, focus, …) can be added
/// when the console → TUI mapping grows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostEvent {
    Resize { width: i64, height: i64 },
    Key(ConsoleKeyEvent),
    Mouse(crate::console_event::ConsoleEvent),
}

impl HostEvent {
    /// Hint for integrating [`TuiSession::request_redraw`]: resize usually implies layout repaint.
    #[must_use]
    pub fn suggests_request_redraw(&self) -> bool {
        matches!(self, Self::Resize { .. })
    }
}

type TraceFn = fn(&str);

/// Coalesces rapid [`TuiEvent::Resize`] bursts and exposes a small ready queue
/// so a key following resizes yields **`Resize` (once, last size) then `Key`**.
#[derive(Debug, Default)]
pub struct TuiHost {
    pending_resize: Option<(i64, i64)>,
    ready: VecDeque<HostEvent>,
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
        match ev {
            TuiEvent::Resize { width, height } => {
                self.trace("tui_host: buffer resize (coalesce)");
                self.pending_resize = Some((width, height));
            }
            TuiEvent::Key(key) => {
                if let Some((width, height)) = self.pending_resize.take() {
                    self.trace("tui_host: flush coalesced resize before key");
                    self.ready.push_back(HostEvent::Resize { width, height });
                }
                self.ready.push_back(HostEvent::Key(key));
            }
            TuiEvent::Mouse(ev) => {
                if let Some((width, height)) = self.pending_resize.take() {
                    self.trace("tui_host: flush coalesced resize before mouse");
                    self.ready.push_back(HostEvent::Resize { width, height });
                }
                self.ready.push_back(HostEvent::Mouse(ev));
            }
        }
    }

    /// Emit a single coalesced [`HostEvent::Resize`] if a resize was buffered and no key arrived yet.
    ///
    /// Intended for an **idle** or **timeout** tick in the outer host loop (see plan Phase 2).
    #[must_use]
    pub fn flush_pending_resize(&mut self) -> bool {
        if let Some((width, height)) = self.pending_resize.take() {
            self.trace("tui_host: flush pending resize (idle)");
            self.ready.push_back(HostEvent::Resize { width, height });
            return true;
        }
        false
    }

    #[must_use]
    pub fn pop_ready_event(&mut self) -> Option<HostEvent> {
        self.ready.pop_front()
    }

    /// Non-blocking: returns a ready [`HostEvent`] or polls the session once.
    pub fn poll_next(
        &mut self,
        session: &TuiSession,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<Option<HostEvent>, StdError> {
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

    /// Blocking: read until at least one [`HostEvent`] is returned (resize-only bursts keep reading).
    pub fn read_next_blocking(
        &mut self,
        session: &TuiSession,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<HostEvent, StdError> {
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
    use crate::console_event::ConsoleEvent;
    use crate::key_event::key_kind_index;

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn coalesces_multiple_resizes_before_key() {
        let mut host = TuiHost::new();
        host.ingest_tui_event(TuiEvent::Resize {
            width: 10,
            height: 10,
        });
        host.ingest_tui_event(TuiEvent::Resize {
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
            Some(HostEvent::Resize {
                width: 20,
                height: 30
            })
        );
        assert!(matches!(host.pop_ready_event(), Some(HostEvent::Key(_))));
        assert_eq!(host.pop_ready_event(), None);
    }

    #[test]
    fn flush_pending_resize_after_resize_only_stream() {
        let mut host = TuiHost::new();
        host.ingest_tui_event(TuiEvent::Resize {
            width: 80,
            height: 25,
        });
        assert!(host.flush_pending_resize());
        assert_eq!(
            host.pop_ready_event(),
            Some(HostEvent::Resize {
                width: 80,
                height: 25
            })
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
            HostEvent::Resize {
                width: 100,
                height: 40
            }
        );
        let second = host
            .read_next_blocking(&session, &mut console, &mut key_input, loc())
            .expect("key");
        assert!(matches!(second, HostEvent::Key(_)));
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
        let r = HostEvent::Resize {
            width: 1,
            height: 1,
        };
        assert!(r.suggests_request_redraw());
        let k = HostEvent::Key(ConsoleKeyEvent::new(
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

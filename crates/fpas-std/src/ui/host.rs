//! Shared hosted-application event queue and resize coalescing.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`, `docs/pascal/std/graph/app/README.md` (from the repository root).

use super::event::{UiEvent, UiResize};
use std::collections::VecDeque;

type TraceFn = fn(&str);

/// Selects which [`UiEvent`] variants a hosted loop accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiHostSurface {
    /// Terminal (`Std.Tui`): keys, mouse, paste, focus, resize.
    Terminal,
    /// Native window (`Std.Graph`): keys, mouse, wheel, close, resize.
    Graph,
}

/// Coalesces rapid [`UiEvent::Resize`] bursts and exposes a ready queue for hosted dispatch.
#[derive(Debug)]
pub struct UiHost {
    surface: UiHostSurface,
    pending_resize: Option<(i64, i64, i64, i64)>,
    ready: VecDeque<UiEvent>,
    trace: Option<TraceFn>,
}

impl UiHost {
    /// Creates one empty terminal host.
    #[must_use]
    pub fn for_terminal() -> Self {
        Self::new(UiHostSurface::Terminal)
    }

    /// Creates one empty graph host.
    #[must_use]
    pub fn for_graph() -> Self {
        Self::new(UiHostSurface::Graph)
    }

    /// Creates one empty host for `surface`.
    #[must_use]
    pub fn new(surface: UiHostSurface) -> Self {
        Self {
            surface,
            pending_resize: None,
            ready: VecDeque::new(),
            trace: None,
        }
    }

    /// Optional structured trace hook (no dependency on `log` / `tracing`).
    pub fn set_trace_hook(&mut self, hook: Option<TraceFn>) {
        self.trace = hook;
    }

    fn trace(&self, msg: &'static str) {
        if let Some(f) = self.trace {
            f(msg);
        }
    }

    /// Feeds one mapped [`UiEvent`] from the session or backend path.
    pub fn ingest_ui_event(&mut self, ev: UiEvent) {
        match ev {
            UiEvent::Resize(UiResize {
                old_width,
                old_height,
                width,
                height,
            }) => {
                self.trace("ui_host: buffer resize (coalesce)");
                let old_w = old_width.unwrap_or(0);
                let old_h = old_height.unwrap_or(0);
                self.pending_resize = Some(
                    self.pending_resize
                        .map_or((old_w, old_h, width, height), |pending| {
                            (pending.0, pending.1, width, height)
                        }),
                );
            }
            UiEvent::Key(key) => {
                self.flush_pending_resize_before("ui_host: flush coalesced resize before key");
                self.ready.push_back(UiEvent::Key(key));
            }
            UiEvent::Mouse(mouse) => {
                self.flush_pending_resize_before("ui_host: flush coalesced resize before mouse");
                self.ready.push_back(UiEvent::Mouse(mouse));
            }
            UiEvent::Wheel(wheel) if self.surface == UiHostSurface::Graph => {
                self.flush_pending_resize_before("ui_host: flush coalesced resize before wheel");
                self.ready.push_back(UiEvent::Wheel(wheel));
            }
            UiEvent::CloseRequested if self.surface == UiHostSurface::Graph => {
                self.flush_pending_resize_before(
                    "ui_host: flush coalesced resize before close-requested",
                );
                self.ready.push_back(UiEvent::CloseRequested);
            }
            UiEvent::Paste(text) if self.surface == UiHostSurface::Terminal => {
                self.flush_pending_resize_before("ui_host: flush coalesced resize before paste");
                self.ready.push_back(UiEvent::Paste(text));
            }
            UiEvent::FocusGained if self.surface == UiHostSurface::Terminal => {
                self.flush_pending_resize_before(
                    "ui_host: flush coalesced resize before focus-gained",
                );
                self.ready.push_back(UiEvent::FocusGained);
            }
            UiEvent::FocusLost if self.surface == UiHostSurface::Terminal => {
                self.flush_pending_resize_before(
                    "ui_host: flush coalesced resize before focus-lost",
                );
                self.ready.push_back(UiEvent::FocusLost);
            }
            UiEvent::CloseRequested
            | UiEvent::Wheel(_)
            | UiEvent::Paste(_)
            | UiEvent::FocusGained
            | UiEvent::FocusLost => {}
        }
    }

    fn flush_pending_resize_before(&mut self, trace_msg: &'static str) {
        self.push_pending_resize(trace_msg);
    }

    /// Emits one coalesced [`UiEvent::Resize`] when a resize burst ended without a following input event.
    #[must_use]
    pub fn flush_pending_resize(&mut self) -> bool {
        self.push_pending_resize("ui_host: flush pending resize (idle)")
    }

    fn push_pending_resize(&mut self, trace_msg: &'static str) -> bool {
        if let Some((old_width, old_height, width, height)) = self.pending_resize.take() {
            self.trace(trace_msg);
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

    /// Removes the next ready event.
    #[must_use]
    pub fn pop_ready_event(&mut self) -> Option<UiEvent> {
        self.ready.pop_front()
    }

    /// Inspects the next ready event without removing it.
    #[must_use]
    pub fn peek_ready_event(&self) -> Option<&UiEvent> {
        self.ready.front()
    }
}

impl Default for UiHost {
    fn default() -> Self {
        Self::for_terminal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConsoleKeyEvent;
    use crate::key_event::key_kind_index;

    #[test]
    fn coalesces_multiple_resizes_before_key() {
        let mut host = UiHost::new(UiHostSurface::Terminal);
        host.ingest_ui_event(UiEvent::Resize(UiResize::new(Some(80), Some(25), 10, 10)));
        host.ingest_ui_event(UiEvent::Resize(UiResize::new(Some(10), Some(10), 20, 30)));
        host.ingest_ui_event(UiEvent::Key(ConsoleKeyEvent::new(
            key_kind_index("Space"),
            ' ',
            false,
            false,
            false,
            false,
        )));

        assert_eq!(
            host.pop_ready_event(),
            Some(UiEvent::Resize(UiResize::new(Some(80), Some(25), 20, 30)))
        );
        assert!(matches!(host.pop_ready_event(), Some(UiEvent::Key(_))));
        assert_eq!(host.pop_ready_event(), None);
    }

    #[test]
    fn graph_accepts_close_and_wheel() {
        let mut host = UiHost::new(UiHostSurface::Graph);
        host.ingest_ui_event(UiEvent::CloseRequested);
        assert_eq!(host.pop_ready_event(), Some(UiEvent::CloseRequested));

        let mut host = UiHost::new(UiHostSurface::Graph);
        host.ingest_ui_event(UiEvent::Wheel(super::super::UiWheel::new(
            0,
            1,
            0,
            0,
            super::super::UiModifiers::default(),
        )));
        assert!(matches!(host.pop_ready_event(), Some(UiEvent::Wheel(_))));
    }

    #[test]
    fn terminal_ignores_close_and_wheel() {
        let mut host = UiHost::new(UiHostSurface::Terminal);
        host.ingest_ui_event(UiEvent::CloseRequested);
        host.ingest_ui_event(UiEvent::Wheel(super::super::UiWheel::new(
            0,
            1,
            0,
            0,
            super::super::UiModifiers::default(),
        )));
        assert_eq!(host.pop_ready_event(), None);
    }
}

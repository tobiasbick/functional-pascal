use crate::console_event::ConsoleEvent;
use crate::key_event::ConsoleKeyEvent;
use crossterm::event::{
    Event, KeyEvent as CrosstermKeyEvent, KeyEventKind, MouseEvent as CrosstermMouseEvent,
};
use crossterm::terminal::disable_raw_mode;
use fpas_bytecode::SourceLocation;
use std::collections::VecDeque;

mod event_source;
mod mapping;
mod read;

#[derive(Debug, Clone)]
pub(super) enum LiveConsoleEvent {
    Key(CrosstermKeyEvent),
    Mouse(CrosstermMouseEvent),
    Resize(u16, u16),
    Paste(String),
    FocusGained,
    FocusLost,
}

/// CRT-style keyboard buffer: test queue plus optional raw-mode stdin via crossterm.
#[derive(Debug, Default)]
pub struct KeyInput {
    test_queue: VecDeque<char>,
    /// Second character of a two-byte extended sequence (`#0` then scan code), CRT-style.
    pending: VecDeque<char>,
    /// Structured events for `ReadKeyEvent` (tests via [`KeyInput::push_key_event`]).
    event_queue: VecDeque<ConsoleKeyEvent>,
    /// Structured events for the unified `ReadEvent` API (tests or live events).
    console_event_queue: VecDeque<ConsoleEvent>,
    /// Real terminal key events captured by non-blocking polling.
    live_queue: VecDeque<CrosstermKeyEvent>,
    /// Non-key terminal events for the unified `ReadEvent` API.
    live_console_queue: VecDeque<LiveConsoleEvent>,
    raw_mode: bool,
    /// Set when any test-queue data is pushed; prevents falling through to live terminal I/O.
    test_mode: bool,
}

impl KeyInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_chars(&mut self, s: &str) {
        self.test_mode = true;
        for c in s.chars() {
            self.test_queue.push_back(c);
        }
    }

    /// Queue a structured key event for the next `ReadKeyEvent` (tests).
    pub fn push_key_event(&mut self, ev: ConsoleKeyEvent) {
        self.test_mode = true;
        self.event_queue.push_back(ev);
    }

    pub fn push_console_event(&mut self, ev: ConsoleEvent) {
        self.test_mode = true;
        self.console_event_queue.push_back(ev);
    }

    /// Queue a live-style event from test code.
    ///
    /// Identical to [`queue_live_event`] but additionally sets `test_mode = true`, so
    /// that once all queued events are consumed, `key_pressed` and `event_pending`
    /// return `false` instead of falling through to live terminal polling.
    #[cfg(test)]
    pub(crate) fn push_live_event(&mut self, ev: Event) -> bool {
        self.test_mode = true;
        self.queue_live_event(ev)
    }

    pub(crate) fn queue_live_event(&mut self, ev: Event) -> bool {
        match ev {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                // Mirror into both API queues so `KeyPressed` and `EventPending` agree,
                // then remove the sibling when either consumer takes the key (see
                // [`Self::take_live_key`] / [`Self::take_live_console_event`]).
                self.live_queue.push_back(key);
                self.live_console_queue
                    .push_back(LiveConsoleEvent::Key(key));
                true
            }
            Event::Mouse(mouse) => {
                self.live_console_queue
                    .push_back(LiveConsoleEvent::Mouse(mouse));
                true
            }
            Event::Resize(width, height) => {
                if let Some(LiveConsoleEvent::Resize(queued_width, queued_height)) =
                    self.live_console_queue.back_mut()
                {
                    *queued_width = width;
                    *queued_height = height;
                } else {
                    self.live_console_queue
                        .push_back(LiveConsoleEvent::Resize(width, height));
                }
                true
            }
            Event::Paste(text) => {
                self.live_console_queue
                    .push_back(LiveConsoleEvent::Paste(text));
                true
            }
            Event::FocusGained => {
                self.live_console_queue
                    .push_back(LiveConsoleEvent::FocusGained);
                true
            }
            Event::FocusLost => {
                self.live_console_queue
                    .push_back(LiveConsoleEvent::FocusLost);
                true
            }
            Event::Key(_) => false,
        }
    }

    /// Pop one live key for `ReadKey` / `ReadKeyEvent`, and drop its mirrored `ReadEvent` entry.
    pub(super) fn take_live_key(&mut self) -> Option<CrosstermKeyEvent> {
        let key = self.live_queue.pop_front()?;
        self.discard_mirrored_live_console_key(&key);
        Some(key)
    }

    /// Pop one live unified event; when it is a key, drop its mirrored `ReadKey` entry.
    pub(super) fn take_live_console_event(&mut self) -> Option<LiveConsoleEvent> {
        let event = self.live_console_queue.pop_front()?;
        if let LiveConsoleEvent::Key(key) = &event {
            self.discard_mirrored_live_key(key);
        }
        Some(event)
    }

    fn discard_mirrored_live_console_key(&mut self, key: &CrosstermKeyEvent) {
        if let Some(pos) = self
            .live_console_queue
            .iter()
            .position(|ev| matches!(ev, LiveConsoleEvent::Key(queued) if queued == key))
        {
            self.live_console_queue.remove(pos);
        }
    }

    fn discard_mirrored_live_key(&mut self, key: &CrosstermKeyEvent) {
        if let Some(pos) = self.live_queue.iter().position(|queued| queued == key) {
            self.live_queue.remove(pos);
        }
    }

    pub fn enable_raw_mode_explicit(
        &mut self,
        location: SourceLocation,
    ) -> Result<(), crate::error::StdError> {
        self.ensure_raw_mode(location)
    }

    pub fn disable_raw_mode_explicit(
        &mut self,
        location: SourceLocation,
    ) -> Result<(), crate::error::StdError> {
        if self.raw_mode {
            disable_raw_mode().map_err(|e| {
                crate::error::std_runtime_error(
                    fpas_diagnostics::codes::RUNTIME_CONSOLE_INPUT_FAILURE,
                    format!("DisableRawMode failed: {e}"),
                    "Run this in an interactive terminal that supports raw mode.",
                    location,
                )
            })?;
            self.raw_mode = false;
        }
        Ok(())
    }
}

impl Drop for KeyInput {
    fn drop(&mut self) {
        if self.raw_mode {
            let _ = disable_raw_mode();
        }
    }
}

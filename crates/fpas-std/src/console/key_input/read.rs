use super::KeyInput;
use super::mapping::{map_console_event, map_crossterm_key, map_key_for_read};
use crate::error::{StdError, std_runtime_error};
use crossterm::event::{self, Event, KeyEvent as CrosstermKeyEvent, KeyEventKind};
use crossterm::terminal::enable_raw_mode;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_INPUT_FAILURE;
use std::time::Duration;

impl KeyInput {
    pub fn key_pressed(&mut self, location: SourceLocation) -> Result<bool, StdError> {
        if !self.pending.is_empty()
            || !self.test_queue.is_empty()
            || !self.event_queue.is_empty()
            || !self.live_queue.is_empty()
        {
            return Ok(true);
        }
        if self.test_mode || !self.raw_mode {
            return Ok(false);
        }
        loop {
            if !event::poll(Duration::ZERO).map_err(|e| {
                std_runtime_error(
                    RUNTIME_CONSOLE_INPUT_FAILURE,
                    format!("KeyPressed failed (poll): {e}"),
                    "Check terminal input availability and try again.",
                    location,
                )
            })? {
                return Ok(false);
            }
            let ev = event::read().map_err(|e| {
                std_runtime_error(
                    RUNTIME_CONSOLE_INPUT_FAILURE,
                    format!("KeyPressed failed (read): {e}"),
                    "Check terminal input availability and try again.",
                    location,
                )
            })?;
            if self.queue_live_event(ev) {
                return Ok(true);
            }
        }
    }

    pub fn read_key(&mut self, location: SourceLocation) -> Result<char, StdError> {
        if let Some(c) = self.pending.pop_front() {
            return Ok(c);
        }
        if let Some(c) = self.test_queue.pop_front() {
            return Ok(c);
        }
        if let Some(key) = self.live_queue.pop_front() {
            return Ok(map_key_for_read(key.code, &mut self.pending));
        }

        self.ensure_raw_mode(location)?;
        let key = self.read_live_key(location, "ReadKey failed")?;
        Ok(map_key_for_read(key.code, &mut self.pending))
    }

    /// One logical key with modifiers (`ReadKeyEvent`); consumes from `event_queue` or `crossterm`.
    pub fn read_key_event(
        &mut self,
        location: SourceLocation,
    ) -> Result<crate::key_event::ConsoleKeyEvent, StdError> {
        if let Some(ev) = self.event_queue.pop_front() {
            return Ok(ev);
        }
        if let Some(key) = self.live_queue.pop_front() {
            return Ok(map_crossterm_key(&key));
        }

        self.ensure_raw_mode(location)?;
        let key = self.read_live_key(location, "ReadKeyEvent failed")?;
        Ok(map_crossterm_key(&key))
    }

    pub fn event_pending(&mut self, location: SourceLocation) -> Result<bool, StdError> {
        if !self.console_event_queue.is_empty() || !self.live_console_queue.is_empty() {
            return Ok(true);
        }
        if self.test_mode || !self.raw_mode {
            return Ok(false);
        }
        loop {
            if !event::poll(Duration::ZERO).map_err(|e| {
                std_runtime_error(
                    RUNTIME_CONSOLE_INPUT_FAILURE,
                    format!("EventPending failed (poll): {e}"),
                    "Check terminal input availability and try again.",
                    location,
                )
            })? {
                return Ok(false);
            }
            let ev = event::read().map_err(|e| {
                std_runtime_error(
                    RUNTIME_CONSOLE_INPUT_FAILURE,
                    format!("EventPending failed (read): {e}"),
                    "Check terminal input availability and try again.",
                    location,
                )
            })?;
            self.queue_live_event(ev);
            if !self.live_console_queue.is_empty() {
                return Ok(true);
            }
        }
    }

    pub fn read_event(
        &mut self,
        location: SourceLocation,
    ) -> Result<crate::console_event::ConsoleEvent, StdError> {
        if let Some(event) = self.console_event_queue.pop_front() {
            return Ok(event);
        }
        if let Some(event) = self.live_console_queue.pop_front() {
            return Ok(map_console_event(event));
        }

        self.ensure_raw_mode(location)?;
        loop {
            let ev = event::read().map_err(|e| {
                std_runtime_error(
                    RUNTIME_CONSOLE_INPUT_FAILURE,
                    format!("ReadEvent failed: {e}"),
                    "Check terminal input and try again.",
                    location,
                )
            })?;
            if self.queue_live_event(ev)
                && let Some(next) = self.live_console_queue.pop_front()
            {
                return Ok(map_console_event(next));
            }
        }
    }

    /// Wait up to `timeout_ms` milliseconds for a console event.
    ///
    /// Returns `Some(event)` if one arrives within the timeout, `None` on timeout.
    pub fn read_event_timeout(
        &mut self,
        timeout_ms: i64,
        location: SourceLocation,
    ) -> Result<Option<crate::console_event::ConsoleEvent>, StdError> {
        // Drain queued events first (test mode or previously buffered).
        if let Some(event) = self.console_event_queue.pop_front() {
            return Ok(Some(event));
        }
        if let Some(event) = self.live_console_queue.pop_front() {
            return Ok(Some(map_console_event(event)));
        }
        if self.test_mode {
            return Ok(None);
        }

        let duration = Duration::from_millis(timeout_ms.max(0) as u64);
        // Raw mode is required for event polling. If not already enabled, return None
        // rather than entering raw mode implicitly. Call EnableRawMode() first.
        if !self.raw_mode {
            return Ok(None);
        }

        if !event::poll(duration).map_err(|e| {
            std_runtime_error(
                RUNTIME_CONSOLE_INPUT_FAILURE,
                format!("ReadEventTimeout failed (poll): {e}"),
                "Check terminal input availability and try again.",
                location,
            )
        })? {
            return Ok(None);
        }

        let ev = event::read().map_err(|e| {
            std_runtime_error(
                RUNTIME_CONSOLE_INPUT_FAILURE,
                format!("ReadEventTimeout failed (read): {e}"),
                "Check terminal input availability and try again.",
                location,
            )
        })?;
        if self.queue_live_event(ev)
            && let Some(next) = self.live_console_queue.pop_front()
        {
            return Ok(Some(map_console_event(next)));
        }
        Ok(None)
    }

    /// Non-blocking event poll — returns `Some(event)` if one is pending, `None` otherwise.
    ///
    /// Unlike `read_event_timeout`, this never enters raw mode; it only checks already-buffered
    /// events and, if already in raw mode, does a 0 ms crossterm poll.
    pub fn poll_event(
        &mut self,
        location: SourceLocation,
    ) -> Result<Option<crate::console_event::ConsoleEvent>, StdError> {
        // Fast path: queued events.
        if let Some(event) = self.console_event_queue.pop_front() {
            return Ok(Some(event));
        }
        if let Some(event) = self.live_console_queue.pop_front() {
            return Ok(Some(map_console_event(event)));
        }
        if self.test_mode {
            return Ok(None);
        }
        // Only poll the real terminal if we are already in raw mode.
        if !self.raw_mode {
            return Ok(None);
        }
        if !event::poll(Duration::ZERO).map_err(|e| {
            std_runtime_error(
                RUNTIME_CONSOLE_INPUT_FAILURE,
                format!("PollEvent failed (poll): {e}"),
                "Check terminal input availability and try again.",
                location,
            )
        })? {
            return Ok(None);
        }
        let ev = event::read().map_err(|e| {
            std_runtime_error(
                RUNTIME_CONSOLE_INPUT_FAILURE,
                format!("PollEvent failed (read): {e}"),
                "Check terminal input availability and try again.",
                location,
            )
        })?;
        if self.queue_live_event(ev)
            && let Some(next) = self.live_console_queue.pop_front()
        {
            return Ok(Some(map_console_event(next)));
        }
        Ok(None)
    }

    pub(super) fn ensure_raw_mode(&mut self, location: SourceLocation) -> Result<(), StdError> {
        if self.raw_mode {
            return Ok(());
        }
        enable_raw_mode().map_err(|e| {
            std_runtime_error(
                RUNTIME_CONSOLE_INPUT_FAILURE,
                format!("ReadKey: raw mode: {e}"),
                "Run this in an interactive terminal that supports raw mode.",
                location,
            )
        })?;
        self.raw_mode = true;
        Ok(())
    }

    fn read_live_key(
        &mut self,
        location: SourceLocation,
        context: &str,
    ) -> Result<CrosstermKeyEvent, StdError> {
        loop {
            match event::read().map_err(|e| {
                std_runtime_error(
                    RUNTIME_CONSOLE_INPUT_FAILURE,
                    format!("{context}: {e}"),
                    "Check terminal input and try again.",
                    location,
                )
            })? {
                Event::Key(key) if key.kind != KeyEventKind::Release => return Ok(key),
                ev => {
                    self.queue_live_event(ev);
                    continue;
                }
            }
        }
    }
}

//! Blocking and polling input operations for an active TUI session.

use super::super::event::{TuiEvent, map_console_event, map_console_ui_event};
use super::TuiSession;
use crate::UiEvent;
use crate::console::{Console, KeyInput};
use crate::error::StdError;
use fpas_bytecode::SourceLocation;
use std::time::{Duration, Instant};

impl TuiSession {
    /// Block until the session yields a supported TUI event.
    pub fn read_event(
        &self,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<TuiEvent, StdError> {
        self.ensure_open(
            "Application.ReadEvent(App) requires an open Std.Tui application session.",
            "Open the application before waiting for events.",
            location,
        )?;

        loop {
            let event = key_input.read_event(location)?;
            if let Some(mapped) = map_console_event(console, event) {
                return Ok(mapped);
            }
        }
    }

    /// Block until the session yields a supported hosted UI event.
    #[doc(hidden)]
    pub fn read_ui_event(
        &self,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<UiEvent, StdError> {
        self.ensure_open(
            "Application.ReadEvent(App) requires an open Std.Tui application session.",
            "Open the application before waiting for events.",
            location,
        )?;

        loop {
            let event = key_input.read_event(location)?;
            if let Some(mapped) = map_console_ui_event(console, event) {
                return Ok(mapped);
            }
        }
    }

    /// Wait up to `timeout_ms` for a supported TUI event.
    pub fn read_event_timeout(
        &self,
        console: &mut Console,
        key_input: &mut KeyInput,
        timeout_ms: i64,
        location: SourceLocation,
    ) -> Result<Option<TuiEvent>, StdError> {
        self.ensure_open(
            "Application.ReadEventTimeout(App, Milliseconds) requires an open Std.Tui application session.",
            "Open the application before waiting for timed events.",
            location,
        )?;

        let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(0) as u64);

        loop {
            let now = Instant::now();
            if now >= deadline {
                return Ok(None);
            }

            let remaining = deadline
                .duration_since(now)
                .as_millis()
                .min(i64::MAX as u128) as i64;

            match key_input.read_event_timeout(remaining, location)? {
                Some(event) => {
                    if let Some(mapped) = map_console_event(console, event) {
                        return Ok(Some(mapped));
                    }
                }
                None => return Ok(None),
            }
        }
    }

    /// Wait up to `timeout_ms` for a supported hosted UI event.
    #[doc(hidden)]
    pub fn read_ui_event_timeout(
        &self,
        console: &mut Console,
        key_input: &mut KeyInput,
        timeout_ms: i64,
        location: SourceLocation,
    ) -> Result<Option<UiEvent>, StdError> {
        self.ensure_open(
            "Application.ReadEventTimeout(App, Milliseconds) requires an open Std.Tui application session.",
            "Open the application before waiting for timed events.",
            location,
        )?;

        let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(0) as u64);

        loop {
            let now = Instant::now();
            if now >= deadline {
                return Ok(None);
            }

            let remaining = deadline
                .duration_since(now)
                .as_millis()
                .min(i64::MAX as u128) as i64;

            match key_input.read_event_timeout(remaining, location)? {
                Some(event) => {
                    if let Some(mapped) = map_console_ui_event(console, event) {
                        return Ok(Some(mapped));
                    }
                }
                None => return Ok(None),
            }
        }
    }

    /// Poll once for a supported TUI event, skipping paste and focus dispatch-only events.
    pub fn poll_event(
        &self,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<Option<TuiEvent>, StdError> {
        self.ensure_open(
            "Application.PollEvent(App) requires an open Std.Tui application session.",
            "Open the application before polling for events.",
            location,
        )?;

        loop {
            match key_input.poll_event(location)? {
                Some(event) => {
                    if let Some(mapped) = map_console_event(console, event) {
                        match mapped {
                            TuiEvent::Paste(_) | TuiEvent::FocusGained | TuiEvent::FocusLost => {
                                continue;
                            }
                            _ => return Ok(Some(mapped)),
                        }
                    }
                }
                None => return Ok(None),
            }
        }
    }

    /// Poll once for a supported hosted UI event, skipping paste and focus dispatch events.
    #[doc(hidden)]
    pub fn poll_ui_event(
        &self,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<Option<UiEvent>, StdError> {
        self.ensure_open(
            "Application.PollEvent(App) requires an open Std.Tui application session.",
            "Open the application before polling for events.",
            location,
        )?;

        loop {
            match key_input.poll_event(location)? {
                Some(event) => {
                    if let Some(mapped) = map_console_ui_event(console, event) {
                        match mapped {
                            UiEvent::Paste(_) | UiEvent::FocusGained | UiEvent::FocusLost => {
                                continue;
                            }
                            _ => return Ok(Some(mapped)),
                        }
                    }
                }
                None => return Ok(None),
            }
        }
    }

    /// Poll once for a supported hosted UI event, including paste and focus dispatch events.
    #[doc(hidden)]
    pub fn poll_ui_event_all(
        &self,
        console: &mut Console,
        key_input: &mut KeyInput,
        location: SourceLocation,
    ) -> Result<Option<UiEvent>, StdError> {
        self.ensure_open(
            "Application.PollEvent(App) requires an open Std.Tui application session.",
            "Open the application before polling for events.",
            location,
        )?;

        loop {
            match key_input.poll_event(location)? {
                Some(event) => {
                    if let Some(mapped) = map_console_ui_event(console, event) {
                        return Ok(Some(mapped));
                    }
                }
                None => return Ok(None),
            }
        }
    }
}

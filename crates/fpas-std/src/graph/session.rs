//! `Std.Graph` session state and validation-oriented Phase 1 helpers.
//!
//! **Documentation:** `docs/future/std.graph/01-mvp.md`, `docs/future/std.graph/02-pascal-surface.md` (from the repository root).

use super::event::GraphEvent;
use super::framebuffer::{UploadedFrame, validate_frame_upload, validate_surface_size};
use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use std::collections::VecDeque;

/// Runtime state for one `Std.Graph` application session.
#[derive(Debug, Default)]
pub struct GraphSession {
    open: bool,
    width: i64,
    height: i64,
    title: String,
    pending_events: VecDeque<GraphEvent>,
    last_uploaded_frame: Option<UploadedFrame>,
}

impl GraphSession {
    /// Opens one graph session after validating the requested surface size.
    pub fn open(
        &mut self,
        width: i64,
        height: i64,
        title: &str,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if self.open {
            return Err(session_state_error(
                "Std.Graph.Application.Open(Width, Height, Title) cannot open a second graphics session while one is already active.",
                "Close the current graphics session with `Application.Close(App)` before opening another one.",
                location,
            ));
        }

        let (width, height) = validate_surface_size(width, height, location)?;
        self.open = true;
        self.width = width;
        self.height = height;
        self.title.clear();
        self.title.push_str(title);
        self.pending_events.clear();
        self.last_uploaded_frame = None;
        Ok(())
    }

    /// Closes the active graph session and clears staged state.
    pub fn close(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.ensure_open(
            "Std.Graph.Application.Close(App) requires an open graphics session.",
            "Open the application with `Application.Open(...)` before closing it.",
            location,
        )?;

        self.open = false;
        self.width = 0;
        self.height = 0;
        self.title.clear();
        self.pending_events.clear();
        self.last_uploaded_frame = None;
        Ok(())
    }

    /// Returns the current logical surface size for the active session.
    pub fn size(&self, location: SourceLocation) -> Result<(i64, i64), StdError> {
        self.ensure_open(
            "Std.Graph.Application.Size(App) requires an open graphics session.",
            "Open the application first and keep the returned handle alive while querying its size.",
            location,
        )?;

        Ok((self.width, self.height))
    }

    /// Polls the next queued graph event.
    pub fn poll_event(
        &mut self,
        location: SourceLocation,
    ) -> Result<Option<GraphEvent>, StdError> {
        self.ensure_open(
            "Std.Graph.Application.PollEvent(App) requires an open graphics session.",
            "Open the application before polling for events.",
            location,
        )?;

        Ok(self.pending_events.pop_front())
    }

    /// Validates and stages one full-frame upload for the active session.
    pub fn upload_frame(
        &mut self,
        width: i64,
        height: i64,
        pixels: &[i64],
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Std.Graph.Application.UploadFrame(App, Width, Height, Pixels) requires an open graphics session.",
            "Open the application before uploading a frame.",
            location,
        )?;

        let validated = validate_frame_upload(self.width, self.height, width, height, pixels, location)?;
        self.last_uploaded_frame = Some(validated);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn push_event_for_tests(&mut self, event: GraphEvent) {
        self.pending_events.push_back(event);
    }

    #[cfg(test)]
    pub(crate) fn last_uploaded_frame_for_tests(&self) -> Option<&UploadedFrame> {
        self.last_uploaded_frame.as_ref()
    }

    fn ensure_open(
        &self,
        message: &'static str,
        help: &'static str,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if self.open {
            Ok(())
        } else {
            Err(session_state_error(message, help, location))
        }
    }
}

fn session_state_error(
    message: &'static str,
    help: &'static str,
    location: SourceLocation,
) -> StdError {
    std_runtime_error(RUNTIME_INTRINSIC_STACK_STATE_ERROR, message, help, location)
}
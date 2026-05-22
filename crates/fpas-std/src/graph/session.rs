//! `Std.Graph` session state and validation-oriented Phase 1 helpers.
//!
//! **Documentation:** `docs/future/std.graph/01-mvp.md`, `docs/future/std.graph/02-pascal-surface.md` (from the repository root).

use super::backbuffer::GraphBackbuffer;
use super::backend;
use super::color::validate_rgb24;
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
    backbuffer: GraphBackbuffer,
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
        let (width, height) = backend::open_graph_backend(width, height, title, location)?;
        let backbuffer = GraphBackbuffer::new(width, height, location)?;
        self.open = true;
        self.width = width;
        self.height = height;
        self.title.clear();
        self.title.push_str(title);
        self.pending_events.clear();
        self.backbuffer = backbuffer;
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

        backend::close_graph_backend(location)?;

        self.open = false;
        self.width = 0;
        self.height = 0;
        self.title.clear();
        self.pending_events.clear();
        self.backbuffer = GraphBackbuffer::default();
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

        backend::graph_surface_size(location)
    }

    /// Polls the next queued graph event.
    pub fn poll_event(&mut self, location: SourceLocation) -> Result<Option<GraphEvent>, StdError> {
        self.ensure_open(
            "Std.Graph.Application.PollEvent(App) requires an open graphics session.",
            "Open the application before polling for events.",
            location,
        )?;

        if let Some(event) = self.pending_events.pop_front() {
            self.apply_polled_event(&event, location)?;
            return Ok(Some(event));
        }

        let event = backend::poll_graph_event(location)?;
        if let Some(event) = &event {
            self.apply_polled_event(event, location)?;
        }

        Ok(event)
    }

    /// Clears the runtime-owned backbuffer with one packed `$00RRGGBB` color.
    pub fn clear(&mut self, color: i64, location: SourceLocation) -> Result<(), StdError> {
        self.ensure_open(
            "Std.Graph.Application.Clear(App, Color) requires an open graphics session.",
            "Open the application before mutating the runtime-owned backbuffer.",
            location,
        )?;

        self.sync_backbuffer_to_backend(location)?;
        let color = validate_rgb24(color, "Std.Graph.Application.Clear(App, Color)", location)?;
        self.backbuffer.clear(color);
        Ok(())
    }

    /// Writes one pixel into the runtime-owned backbuffer and clips out-of-bounds coordinates.
    pub fn put_pixel(
        &mut self,
        x: i64,
        y: i64,
        color: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Std.Graph.Application.PutPixel(App, X, Y, Color) requires an open graphics session.",
            "Open the application before mutating the runtime-owned backbuffer.",
            location,
        )?;

        self.sync_backbuffer_to_backend(location)?;
        let color = validate_rgb24(
            color,
            "Std.Graph.Application.PutPixel(App, X, Y, Color)",
            location,
        )?;
        self.backbuffer.put_pixel(x, y, color);
        Ok(())
    }

    /// Presents the current runtime-owned backbuffer to the native window.
    pub fn present(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.ensure_open(
            "Std.Graph.Application.Present(App) requires an open graphics session.",
            "Open the application before presenting the runtime-owned backbuffer.",
            location,
        )?;

        self.sync_backbuffer_to_backend(location)?;
        let frame = self.backbuffer.snapshot();
        backend::present_graph_frame(&frame, location)?;
        Ok(())
    }

    /// Queues one normalized host event for the active session.
    pub fn push_event(
        &mut self,
        event: GraphEvent,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Std.Graph host event injection requires an open graphics session.",
            "Open the application before queueing host events for it.",
            location,
        )?;

        self.pending_events.push_back(event);
        Ok(())
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

        let (expected_width, expected_height) = backend::graph_surface_size(location)?;
        self.width = expected_width;
        self.height = expected_height;
        let validated = validate_frame_upload(
            expected_width,
            expected_height,
            width,
            height,
            pixels,
            location,
        )?;
        self.backbuffer.overwrite(&validated, location)?;
        backend::present_graph_frame(&validated, location)?;
        self.last_uploaded_frame = Some(validated);
        Ok(())
    }

    /// Returns the most recently validated frame upload, if one is staged.
    pub fn uploaded_frame(&self) -> Option<&UploadedFrame> {
        self.last_uploaded_frame.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn backbuffer_pixels_for_tests(&self) -> &[u32] {
        self.backbuffer.pixels()
    }

    #[cfg(test)]
    pub(crate) fn backbuffer_size_for_tests(&self) -> (i64, i64) {
        self.backbuffer.size()
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

    fn sync_backbuffer_to_backend(&mut self, location: SourceLocation) -> Result<(), StdError> {
        let (width, height) = backend::graph_surface_size(location)?;
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            self.backbuffer.resize(width, height, location)?;
        }
        Ok(())
    }

    fn apply_polled_event(
        &mut self,
        event: &GraphEvent,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if let GraphEvent::Resize { width, height } = event {
            self.width = *width;
            self.height = *height;
            self.backbuffer.resize(*width, *height, location)?;
        }
        Ok(())
    }
}

fn session_state_error(
    message: &'static str,
    help: &'static str,
    location: SourceLocation,
) -> StdError {
    std_runtime_error(RUNTIME_INTRINSIC_STACK_STATE_ERROR, message, help, location)
}

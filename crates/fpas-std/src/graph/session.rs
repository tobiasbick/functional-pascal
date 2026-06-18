//! `Std.Graph` session state and validation-oriented Phase 1 helpers.
//!
//! **Documentation:** `docs/pascal/std/graph/session.md` (from the repository root).

use super::backbuffer::GraphBackbuffer;
use super::backend;
use super::circle;
use super::color::validate_rgb24;
use super::event::GraphEvent;
use super::framebuffer::{UploadedFrame, validate_frame_upload, validate_surface_size};
use super::line;
use super::rect;
use super::text;
use crate::error::{StdError, std_runtime_error};
use crate::ui::{UiEvent, UiResize};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use std::collections::VecDeque;

/// Runtime state for one `Std.Graph` application session.
#[derive(Debug, Default)]
pub struct GraphSession {
    open: bool,
    width: i64,
    height: i64,
    pending_events: VecDeque<UiEvent>,
    backbuffer: GraphBackbuffer,
    last_uploaded_frame: Option<UploadedFrame>,
    redraw_pending: bool,
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
        self.pending_events.clear();
        self.backbuffer = backbuffer;
        self.last_uploaded_frame = None;
        self.redraw_pending = false;
        Ok(())
    }

    /// Closes the active graph session and clears staged state.
    pub fn close(&mut self, location: SourceLocation) -> Result<(), StdError> {
        if !self.open {
            return Ok(());
        }

        backend::close_graph_backend(location)?;

        self.open = false;
        self.width = 0;
        self.height = 0;
        self.pending_events.clear();
        self.backbuffer = GraphBackbuffer::default();
        self.last_uploaded_frame = None;
        self.redraw_pending = false;
        Ok(())
    }

    /// Returns whether a graph session is currently open.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Returns the current logical surface size for the active session.
    pub fn size(&mut self, location: SourceLocation) -> Result<(i64, i64), StdError> {
        self.ensure_open(
            "Std.Graph.Application.Size(App) requires an open graphics session.",
            "Open the application first and keep the returned handle alive while querying its size.",
            location,
        )?;

        self.sync_backbuffer_to_backend(location)?;
        Ok((self.width, self.height))
    }

    /// Marks the active session as needing a hosted redraw.
    pub fn request_redraw(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.ensure_open(
            "Application.RequestRedraw(App) requires an open graphics session.",
            "Open the application before requesting a redraw.",
            location,
        )?;
        self.redraw_pending = true;
        Ok(())
    }

    /// Marks the active session as needing a hosted redraw when none is already pending.
    pub fn request_redraw_if_absent(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.ensure_open(
            "Application.RequestRedraw(App) requires an open graphics session.",
            "Open the application before requesting a redraw.",
            location,
        )?;
        if !self.redraw_pending {
            self.redraw_pending = true;
        }
        Ok(())
    }

    /// Returns whether a hosted redraw is pending without consuming it.
    pub fn peek_redraw_pending(&self, location: SourceLocation) -> Result<bool, StdError> {
        self.ensure_open(
            "Hosted graph redraw requires an open graphics session.",
            "Open the application before querying redraw state.",
            location,
        )?;
        Ok(self.redraw_pending)
    }

    /// Consumes and returns whether a hosted redraw was pending.
    pub fn take_redraw_pending(&mut self, location: SourceLocation) -> Result<bool, StdError> {
        self.ensure_open(
            "Hosted graph redraw requires an open graphics session.",
            "Open the application before consuming redraw state.",
            location,
        )?;
        let pending = self.redraw_pending;
        self.redraw_pending = false;
        Ok(pending)
    }

    /// Waits up to `timeout_ms` for the next hosted UI event from the native backend.
    pub fn read_host_ui_event_timeout(
        &mut self,
        timeout_ms: i64,
        location: SourceLocation,
    ) -> Result<Option<UiEvent>, StdError> {
        self.ensure_open(
            "Std.Graph hosted event wait requires an open graphics session.",
            "Open the application before waiting for events.",
            location,
        )?;

        if let Some(event) = self.pending_events.pop_front() {
            self.apply_polled_event(&event, location)?;
            return Ok(Some(event));
        }

        let event = backend::read_graph_event_timeout(timeout_ms, location)?;
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

    /// Draws one clipped line into the runtime-owned backbuffer.
    pub fn draw_line(
        &mut self,
        x1: i64,
        y1: i64,
        x2: i64,
        y2: i64,
        color: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Std.Graph.Application.DrawLine(App, X1, Y1, X2, Y2, Color) requires an open graphics session.",
            "Open the application before mutating the runtime-owned backbuffer.",
            location,
        )?;

        self.sync_backbuffer_to_backend(location)?;
        let color = validate_rgb24(
            color,
            "Std.Graph.Application.DrawLine(App, X1, Y1, X2, Y2, Color)",
            location,
        )?;
        line::draw_line(&mut self.backbuffer, x1, y1, x2, y2, color);
        Ok(())
    }

    /// Draws one clipped rectangle outline into the runtime-owned backbuffer.
    pub fn draw_rect(
        &mut self,
        x: i64,
        y: i64,
        width: i64,
        height: i64,
        color: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Std.Graph.Application.DrawRect(App, X, Y, Width, Height, Color) requires an open graphics session.",
            "Open the application before mutating the runtime-owned backbuffer.",
            location,
        )?;

        self.sync_backbuffer_to_backend(location)?;
        let color = validate_rgb24(
            color,
            "Std.Graph.Application.DrawRect(App, X, Y, Width, Height, Color)",
            location,
        )?;
        rect::draw_rect(&mut self.backbuffer, x, y, width, height, color, location)
    }

    /// Fills one clipped rectangle into the runtime-owned backbuffer.
    pub fn fill_rect(
        &mut self,
        x: i64,
        y: i64,
        width: i64,
        height: i64,
        color: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Std.Graph.Application.FillRect(App, X, Y, Width, Height, Color) requires an open graphics session.",
            "Open the application before mutating the runtime-owned backbuffer.",
            location,
        )?;

        self.sync_backbuffer_to_backend(location)?;
        let color = validate_rgb24(
            color,
            "Std.Graph.Application.FillRect(App, X, Y, Width, Height, Color)",
            location,
        )?;
        rect::fill_rect(&mut self.backbuffer, x, y, width, height, color, location)
    }

    /// Draws one clipped circle outline into the runtime-owned backbuffer.
    pub fn draw_circle(
        &mut self,
        center_x: i64,
        center_y: i64,
        radius: i64,
        color: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Std.Graph.Application.DrawCircle(App, CenterX, CenterY, Radius, Color) requires an open graphics session.",
            "Open the application before mutating the runtime-owned backbuffer.",
            location,
        )?;

        self.sync_backbuffer_to_backend(location)?;
        let color = validate_rgb24(
            color,
            "Std.Graph.Application.DrawCircle(App, CenterX, CenterY, Radius, Color)",
            location,
        )?;
        circle::draw_circle(
            &mut self.backbuffer,
            center_x,
            center_y,
            radius,
            color,
            location,
        )
    }

    /// Draws deterministic bitmap text into the runtime-owned backbuffer.
    pub fn draw_text(
        &mut self,
        x: i64,
        y: i64,
        text_value: &str,
        color: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Std.Graph.Application.DrawText(App, X, Y, Text, Color) requires an open graphics session.",
            "Open the application before mutating the runtime-owned backbuffer.",
            location,
        )?;

        self.sync_backbuffer_to_backend(location)?;
        let color = validate_rgb24(
            color,
            "Std.Graph.Application.DrawText(App, X, Y, Text, Color)",
            location,
        )?;
        text::draw_text(&mut self.backbuffer, x, y, text_value, color);
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

        self.pending_events.push_back(event.into_ui_event());
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

        let (expected_width, expected_height) = (self.width, self.height);
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
        self.pending_events.push_back(event.into_ui_event());
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
        event: &UiEvent,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if let UiEvent::Resize(UiResize { width, height, .. }) = event {
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

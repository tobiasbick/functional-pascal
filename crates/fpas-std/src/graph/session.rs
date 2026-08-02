//! `Std.Graph` session state and validation-oriented Phase 1 helpers.
//!
//! **Documentation:** `docs/pascal/std/graph/session.md` (from the repository root).

use super::backbuffer::GraphBackbuffer;
use super::backend;
use super::circle;
use super::color::validate_rgb24;
use super::framebuffer::{UploadedFrame, validate_frame_upload, validate_surface_size};
use super::line;
use super::rect;
use super::text;
use crate::error::{StdError, std_runtime_error};
use crate::ui::UiEvent;
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use std::collections::VecDeque;

mod events;

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
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid surface size, a second active session,
    /// or a backend initialization failure.
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
    ///
    /// The session becomes closed even when backend teardown reports an error,
    /// because the thread-local backend resource has already been detached.
    ///
    /// # Errors
    ///
    /// Returns an error when the active backend cannot complete its teardown.
    pub fn close(&mut self, location: SourceLocation) -> Result<(), StdError> {
        if !self.open {
            return Ok(());
        }

        let result = backend::close_graph_backend(location);
        self.clear_session_state();
        result
    }

    fn clear_session_state(&mut self) {
        self.open = false;
        self.width = 0;
        self.height = 0;
        self.pending_events.clear();
        self.backbuffer = GraphBackbuffer::default();
        self.last_uploaded_frame = None;
        self.redraw_pending = false;
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
}

impl Drop for GraphSession {
    fn drop(&mut self) {
        if self.open {
            let _ = backend::close_graph_backend(SourceLocation::new(1, 1));
            self.clear_session_state();
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

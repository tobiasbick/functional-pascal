//! `Std.Graph` framebuffer validation helpers.
//!
//! **Documentation:** `docs/future/std.graph/02-pascal-surface.md` (from the repository root).

use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

const MAX_RGB24: i64 = 0x00FF_FFFF;

/// Validated frame payload ready for a future backend upload path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadedFrame {
    pub(crate) width: i64,
    pub(crate) height: i64,
    pub(crate) pixels: Vec<u32>,
}

impl UploadedFrame {
    /// Returns the validated frame width in pixels.
    pub fn width(&self) -> i64 {
        self.width
    }

    /// Returns the validated frame height in pixels.
    pub fn height(&self) -> i64 {
        self.height
    }

    /// Returns the validated packed `$00RRGGBB` pixel payload.
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }
}

/// Validates positive surface dimensions for `Std.Graph.Application.Open`.
pub(crate) fn validate_surface_size(
    width: i64,
    height: i64,
    location: SourceLocation,
) -> Result<(i64, i64), StdError> {
    if width <= 0 || height <= 0 {
        return Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "Std.Graph.Application.Open(Width, Height, Title) requires positive dimensions, got Width={width} and Height={height}."
            ),
            "Pass positive pixel dimensions such as `Application.Open(640, 480, 'Graph')`.",
            location,
        ));
    }

    Ok((width, height))
}

/// Validates one full-frame upload against the current session size.
pub(crate) fn validate_frame_upload(
    expected_width: i64,
    expected_height: i64,
    width: i64,
    height: i64,
    pixels: &[i64],
    location: SourceLocation,
) -> Result<UploadedFrame, StdError> {
    let (width, height) = validate_surface_size(width, height, location)?;

    if width != expected_width || height != expected_height {
        return Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "Std.Graph.Application.UploadFrame(App, Width, Height, Pixels) expected Width={expected_width} and Height={expected_height}, got Width={width} and Height={height}."
            ),
            "Use `Application.Size(App)` and pass those exact dimensions to `Application.UploadFrame`.",
            location,
        ));
    }

    let expected_len = width.saturating_mul(height);
    let got_len = i64::try_from(pixels.len()).unwrap_or(i64::MAX);
    if got_len != expected_len {
        return Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "Std.Graph.Application.UploadFrame(App, Width, Height, Pixels) expected {expected_len} pixels for {width}x{height}, got {got_len}."
            ),
            "Ensure `Length(Pixels) = Width * Height` and that the frame is row-major.",
            location,
        ));
    }

    let mut converted = Vec::with_capacity(pixels.len());
    for (index, &pixel) in pixels.iter().enumerate() {
        if !(0..=MAX_RGB24).contains(&pixel) {
            return Err(std_runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!(
                    "Std.Graph.Application.UploadFrame(App, Width, Height, Pixels) requires `$00RRGGBB` pixels; Pixels[{index}] = {pixel} is out of range."
                ),
                "Store each pixel as an integer between 0 and 16777215 (`$00RRGGBB`).",
                location,
            ));
        }
        converted.push(u32::try_from(pixel).unwrap_or_default());
    }

    Ok(UploadedFrame {
        width,
        height,
        pixels: converted,
    })
}
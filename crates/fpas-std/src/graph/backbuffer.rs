//! `Std.Graph` runtime-owned backbuffer storage.
//!
//! **Documentation:** `docs/pascal/std/graph/session.md` (from the repository root).

use super::framebuffer::UploadedFrame;
use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct GraphBackbuffer {
    width: i64,
    height: i64,
    pixels: Vec<u32>,
}

impl GraphBackbuffer {
    pub(crate) fn new(width: i64, height: i64, location: SourceLocation) -> Result<Self, StdError> {
        let len = pixel_count(width, height, location)?;
        Ok(Self {
            width,
            height,
            pixels: vec![0; len],
        })
    }

    pub(crate) fn resize(
        &mut self,
        width: i64,
        height: i64,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if self.width == width && self.height == height {
            return Ok(());
        }

        let len = pixel_count(width, height, location)?;
        self.width = width;
        self.height = height;
        self.pixels.clear();
        self.pixels.resize(len, 0);
        Ok(())
    }

    pub(crate) fn clear(&mut self, color: u32) {
        self.pixels.fill(color);
    }

    pub(crate) fn put_pixel(&mut self, x: i64, y: i64, color: u32) {
        if !(0..self.width).contains(&x) || !(0..self.height).contains(&y) {
            return;
        }

        let row = usize::try_from(y).unwrap_or_default();
        let col = usize::try_from(x).unwrap_or_default();
        let width = usize::try_from(self.width).unwrap_or_default();
        let index = row.saturating_mul(width).saturating_add(col);
        if let Some(slot) = self.pixels.get_mut(index) {
            *slot = color;
        }
    }

    pub(crate) fn overwrite(
        &mut self,
        frame: &UploadedFrame,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.resize(frame.width(), frame.height(), location)?;
        self.pixels.clear();
        self.pixels.extend_from_slice(frame.pixels());
        Ok(())
    }

    pub(crate) fn snapshot(&self) -> UploadedFrame {
        UploadedFrame {
            width: self.width,
            height: self.height,
            pixels: self.pixels.clone(),
        }
    }

    #[cfg(test)]
    pub(crate) fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    #[cfg(test)]
    pub(crate) fn size(&self) -> (i64, i64) {
        (self.width, self.height)
    }
}

fn pixel_count(width: i64, height: i64, location: SourceLocation) -> Result<usize, StdError> {
    let width_usize =
        usize::try_from(width).map_err(|_| allocation_error(width, height, location))?;
    let height_usize =
        usize::try_from(height).map_err(|_| allocation_error(width, height, location))?;
    let len = width_usize
        .checked_mul(height_usize)
        .ok_or_else(|| allocation_error(width, height, location))?;
    if len > crate::limits::MAX_GRAPH_PIXELS {
        return Err(allocation_error(width, height, location));
    }
    Ok(len)
}

fn allocation_error(width: i64, height: i64, location: SourceLocation) -> StdError {
    std_runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!(
            "Std.Graph cannot allocate a runtime backbuffer for Width={width} and Height={height}."
        ),
        "Reduce the requested surface size so the runtime can allocate `Width * Height` pixels.",
        location,
    )
}

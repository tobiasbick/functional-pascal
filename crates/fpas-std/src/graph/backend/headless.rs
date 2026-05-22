//! Deterministic headless `Std.Graph` backend used by automated tests.
//!
//! **Documentation:** `docs/future/std.graph/04-implementation-plan.md` (from the repository root).

use super::super::{GraphEvent, UploadedFrame};
use crate::error::StdError;
use fpas_bytecode::SourceLocation;

/// Headless graph backend that mirrors size and present operations without opening a window.
#[derive(Debug)]
pub(crate) struct HeadlessGraphBackend {
    width: i64,
    height: i64,
}

impl HeadlessGraphBackend {
    /// Creates one headless backend instance for automated tests.
    pub(crate) fn open(width: i64, height: i64, title: &str) -> Self {
        let _ = title;
        Self { width, height }
    }

    /// Closes the headless backend.
    pub(crate) fn close(&mut self, location: SourceLocation) -> Result<(), StdError> {
        let _ = location;
        Ok(())
    }

    /// Polls the next queued event from the headless backend.
    pub(crate) fn poll_event(
        &mut self,
        location: SourceLocation,
    ) -> Result<Option<GraphEvent>, StdError> {
        let _ = location;
        Ok(None)
    }

    /// Accepts a validated frame without presenting it anywhere.
    pub(crate) fn present_frame(
        &mut self,
        frame: &UploadedFrame,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        let _ = location;
        self.width = frame.width();
        self.height = frame.height();
        Ok(())
    }

    /// Returns the current logical surface size.
    pub(crate) fn size(&mut self, location: SourceLocation) -> Result<(i64, i64), StdError> {
        let _ = location;
        Ok((self.width, self.height))
    }
}

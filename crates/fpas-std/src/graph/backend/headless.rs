//! Deterministic headless `Std.Graph` backend used by automated tests.
//!
//! **Documentation:** `docs/pascal/std/graph/session.md` (from the repository root).

use super::super::UploadedFrame;
use crate::error::StdError;
use crate::ui::UiEvent;
use fpas_bytecode::SourceLocation;
use std::cell::RefCell;

thread_local! {
    static LAST_PRESENTED_FRAME: RefCell<Option<UploadedFrame>> = const { RefCell::new(None) };
}

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
        reset_last_presented_frame_for_tests();
        Self { width, height }
    }

    /// Closes the headless backend.
    pub(crate) fn close(&mut self, location: SourceLocation) -> Result<(), StdError> {
        let _ = location;
        Ok(())
    }

    /// Waits up to `timeout_ms` milliseconds for the next queued event.
    pub(crate) fn read_event_timeout(
        &mut self,
        timeout_ms: i64,
        location: SourceLocation,
    ) -> Result<Option<UiEvent>, StdError> {
        let _ = timeout_ms;
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
        LAST_PRESENTED_FRAME.with(|slot| {
            *slot.borrow_mut() = Some(frame.clone());
        });
        Ok(())
    }

    /// Returns the current logical surface size.
    pub(crate) fn size(&mut self, location: SourceLocation) -> Result<(i64, i64), StdError> {
        let _ = location;
        Ok((self.width, self.height))
    }

    #[cfg(test)]
    pub(crate) fn set_size_for_tests(&mut self, width: i64, height: i64) {
        self.width = width;
        self.height = height;
    }
}

pub(super) fn last_presented_frame_for_tests() -> Option<UploadedFrame> {
    LAST_PRESENTED_FRAME.with(|slot| slot.borrow().clone())
}

pub(super) fn reset_last_presented_frame_for_tests() {
    LAST_PRESENTED_FRAME.with(|slot| {
        slot.borrow_mut().take();
    });
}

//! Send-safe boolean storage for Turbo Vision `CheckBox` handles.
//!
//! Modal dialog views sync edits back into the shared cell after user interaction.

use std::sync::{Arc, Mutex};

/// Shared check-box state owned by the FPAS Turbo Vision bridge.
#[derive(Clone, Debug)]
pub(crate) struct TurboVisionBoolCell(Arc<Mutex<bool>>);

impl TurboVisionBoolCell {
    /// Create a cell with the given initial value.
    pub fn new(value: bool) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }

    /// Read the current host-side value.
    pub fn read(&self) -> bool {
        *self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Replace the host-side value.
    pub fn set(&self, value: bool) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = value;
    }
}

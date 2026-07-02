//! Send-safe selection storage for Turbo Vision `ListBox` handles.
//!
//! Modal dialog views sync selection back into the shared cell after user interaction.

use std::sync::{Arc, Mutex};

/// Shared list-box selection owned by the FPAS Turbo Vision bridge.
#[derive(Clone, Debug)]
pub(crate) struct TurboVisionListSelectionCell(Arc<Mutex<Option<usize>>>);

impl TurboVisionListSelectionCell {
    /// Create a cell with the given initial selection.
    pub fn new(selection: Option<usize>) -> Self {
        Self(Arc::new(Mutex::new(selection)))
    }

    /// Read the current host-side selection.
    pub fn read(&self) -> Option<usize> {
        *self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Replace the host-side selection.
    pub fn set(&self, selection: Option<usize>) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = selection;
    }
}

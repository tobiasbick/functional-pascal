//! Send-safe text storage for Turbo Vision `InputLine` handles.
//!
//! Host state keeps an `Arc<Mutex<String>>`. Views bind through a short-lived
//! `Rc<RefCell<String>>` registered on the main [`Worker`](crate::vm::Worker) at
//! desktop populate time so `SetText` can mirror into the live view buffer.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Shared input-line text owned by the FPAS Turbo Vision bridge.
#[derive(Clone, Debug)]
pub(crate) struct TurboVisionInputTextCell(Arc<Mutex<String>>);

impl TurboVisionInputTextCell {
    /// Create a cell with the given initial text.
    pub fn new(text: String) -> Self {
        Self(Arc::new(Mutex::new(text)))
    }

    /// Read the current host-side text.
    pub fn read(&self) -> String {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Replace the host-side text.
    pub fn set(&self, text: String) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = text;
    }

    /// Create a view binding seeded from the current host text.
    pub fn view_binding(&self) -> Rc<RefCell<String>> {
        Rc::new(RefCell::new(self.read()))
    }

    /// Copy edited view text back into the host cell.
    pub fn commit_view_binding(&self, binding: &Rc<RefCell<String>>) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = binding.borrow().clone();
    }
}

//! Send-safe text storage for Turbo Vision `InputLine` handles.
//!
//! Host state keeps an `Arc<Mutex<String>>`. Views bind through a short-lived
//! `Rc<RefCell<String>>` that is copied back after modal `execute`.

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

    /// Create a view binding for upstream `InputLine::new`.
    pub fn view_binding(&self) -> Rc<RefCell<String>> {
        Rc::new(RefCell::new(self.read()))
    }

    /// Copy edited view text back into the host cell.
    pub fn commit_view_binding(&self, binding: &Rc<RefCell<String>>) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = binding.borrow().clone();
    }
}

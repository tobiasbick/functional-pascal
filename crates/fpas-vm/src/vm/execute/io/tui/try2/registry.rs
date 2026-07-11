//! FPAS view handle registry for try-2 (index into live turbo-vision `ViewId`s).
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-architecture.md`

#![allow(dead_code, reason = "try-2 scaffold; Worker wiring lands in phase 1/2")]

use std::collections::HashMap;

/// Widget category for handle validation and diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewKind {
    Application,
    Dialog,
    Window,
    EditorWindow,
    Button,
    StaticText,
    InputLine,
    ListBox,
    CheckBox,
    RadioButton,
    Memo,
    TextViewer,
    Outline,
    MenuBar,
    StatusLine,
}

/// Live upstream view reference stored for a FPAS-allocated handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TvViewRef {
    /// Upstream `ViewId` encoded as `u16` (see turbo-vision `ViewId::as_u16`).
    pub view_id: u16,
    pub kind: ViewKind,
}

/// Maps opaque FPAS view handles to upstream view ids.
#[derive(Debug, Default)]
pub struct ViewRegistry {
    next_id: u32,
    entries: HashMap<u32, TvViewRef>,
}

/// Handle validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    UnknownHandle(u32),
    WrongKind {
        handle: u32,
        expected: ViewKind,
        actual: ViewKind,
    },
}

impl ViewRegistry {
    /// Empty registry; FPAS application id `0` is reserved and never allocated.
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_id: 1,
            entries: HashMap::new(),
        }
    }

    /// Register a live upstream view and return a new FPAS handle.
    pub fn allocate(&mut self, view_id: u16, kind: ViewKind) -> u32 {
        let handle = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.entries.insert(handle, TvViewRef { view_id, kind });
        handle
    }

    /// Look up a handle without kind checking.
    #[must_use]
    pub fn get(&self, handle: u32) -> Option<&TvViewRef> {
        self.entries.get(&handle)
    }

    /// Remove a handle; returns the former entry if it existed.
    pub fn remove(&mut self, handle: u32) -> Option<TvViewRef> {
        self.entries.remove(&handle)
    }

    /// Updates the upstream view id for an existing handle (detached → attached).
    pub fn set_view_id(&mut self, handle: u32, view_id: u16) -> Result<(), RegistryError> {
        let Some(entry) = self.entries.get_mut(&handle) else {
            return Err(RegistryError::UnknownHandle(handle));
        };
        entry.view_id = view_id;
        Ok(())
    }

    /// Drop all entries (session close).
    pub fn clear(&mut self) {
        self.entries.clear();
        self.next_id = 1;
    }

    /// Number of live handles (tests and diagnostics).
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Require a live handle of the expected widget kind.
    pub fn require(&self, handle: u32, expected: ViewKind) -> Result<&TvViewRef, RegistryError> {
        let Some(entry) = self.entries.get(&handle) else {
            return Err(RegistryError::UnknownHandle(handle));
        };
        if entry.kind != expected {
            return Err(RegistryError::WrongKind {
                handle,
                expected,
                actual: entry.kind,
            });
        }
        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_and_lookup() {
        let mut registry = ViewRegistry::new();
        let h = registry.allocate(42, ViewKind::Button);
        assert_eq!(registry.get(h).unwrap().view_id, 42);
        assert_eq!(registry.get(h).unwrap().kind, ViewKind::Button);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn require_kind_mismatch() {
        let mut registry = ViewRegistry::new();
        let h = registry.allocate(1, ViewKind::Dialog);
        assert!(matches!(
            registry.require(h, ViewKind::Button),
            Err(RegistryError::WrongKind { .. })
        ));
    }

    #[test]
    fn set_view_id_updates_entry() {
        let mut registry = ViewRegistry::new();
        let h = registry.allocate(0, ViewKind::Button);
        registry.set_view_id(h, 7).expect("set");
        assert_eq!(registry.get(h).unwrap().view_id, 7);
    }

    #[test]
    fn unknown_handle() {
        let registry = ViewRegistry::new();
        assert!(matches!(
            registry.require(99, ViewKind::Window),
            Err(RegistryError::UnknownHandle(99))
        ));
    }

    #[test]
    fn clear_resets_handles() {
        let mut registry = ViewRegistry::new();
        let _ = registry.allocate(1, ViewKind::Window);
        registry.clear();
        assert!(registry.is_empty());
        let h = registry.allocate(2, ViewKind::Button);
        assert_eq!(h, 1);
    }
}

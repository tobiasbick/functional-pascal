//! Rust-internal modal stack for the TUI application framework (Phase 7).
//!
//! The Pascal-facing host API is documented in `docs/pascal/std/tui-app.md`.

/// Application-defined modal identifier supplied by FPAS code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModalId(pub i64);

/// Host-side modal stack for an active TUI session.
#[derive(Debug, Default)]
pub struct ModalStack {
    ids: Vec<ModalId>,
}

impl ModalStack {
    /// Push `modal_id` as the active modal.
    pub fn enter(&mut self, modal_id: ModalId) {
        self.ids.push(modal_id);
    }

    /// Pop the active modal, if any.
    pub fn leave(&mut self) -> Option<ModalId> {
        self.ids.pop()
    }

    /// Active modal id, if a modal is active.
    #[must_use]
    pub fn active_id(&self) -> Option<ModalId> {
        self.ids.last().copied()
    }

    /// Number of active modal frames.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.ids.len()
    }

    /// Remove all active modal frames.
    pub fn clear(&mut self) {
        self.ids.clear();
    }

    /// True when no modal frame is active.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_tracks_depth_and_active_id() {
        let mut modals = ModalStack::default();

        modals.enter(ModalId(10));
        modals.enter(ModalId(20));

        assert_eq!(modals.depth(), 2);
        assert_eq!(modals.active_id(), Some(ModalId(20)));
    }

    #[test]
    fn leave_pops_last_entered_modal() {
        let mut modals = ModalStack::default();
        modals.enter(ModalId(10));
        modals.enter(ModalId(20));

        assert_eq!(modals.leave(), Some(ModalId(20)));
        assert_eq!(modals.active_id(), Some(ModalId(10)));
        assert_eq!(modals.depth(), 1);
    }

    #[test]
    fn leave_empty_stack_is_noop() {
        let mut modals = ModalStack::default();

        assert_eq!(modals.leave(), None);
        assert_eq!(modals.depth(), 0);
    }

    #[test]
    fn clear_removes_all_modal_frames() {
        let mut modals = ModalStack::default();
        modals.enter(ModalId(10));
        modals.enter(ModalId(20));

        modals.clear();

        assert!(modals.is_empty());
    }
}

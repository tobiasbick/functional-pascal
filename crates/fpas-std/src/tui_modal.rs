//! Rust-internal modal stack for the TUI application framework (Phase 7).
//!
//! The Pascal-facing host API is documented in `docs/pascal/std/tui-app.md`.

use crate::ViewId;

/// Application-defined modal identifier supplied by FPAS code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModalId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModalFrame {
    id: ModalId,
    root_view: Option<ViewId>,
    scoped_views: Vec<ViewId>,
}

/// Host-side modal stack for an active TUI session.
#[derive(Debug, Default)]
pub struct ModalStack {
    frames: Vec<ModalFrame>,
}

impl ModalStack {
    /// Push `modal_id` as the active modal.
    pub fn enter(&mut self, modal_id: ModalId) {
        self.frames.push(ModalFrame {
            id: modal_id,
            root_view: None,
            scoped_views: Vec::new(),
        });
    }

    /// Push `modal_id` as the active modal and bind it to `root_view`.
    ///
    /// The host uses the root view's full subtree as the modal scope for focus, mouse routing,
    /// and local paint ordering.
    pub fn show(&mut self, modal_id: ModalId, root_view: ViewId) {
        self.frames.push(ModalFrame {
            id: modal_id,
            root_view: Some(root_view),
            scoped_views: Vec::new(),
        });
    }

    /// Pop the active modal, if any.
    pub fn leave(&mut self) -> Option<ModalId> {
        self.pop_frame().map(|frame| frame.id)
    }

    /// Pop the active modal and return its id plus the views attached to that frame.
    #[must_use]
    pub fn leave_with_scoped_views(&mut self) -> Option<(ModalId, Vec<ViewId>)> {
        self.pop_frame().map(|frame| (frame.id, frame.scoped_views))
    }

    /// Pop the active modal and return its id, optional root view, and manually attached views.
    #[must_use]
    pub fn leave_with_scope_info(&mut self) -> Option<(ModalId, Option<ViewId>, Vec<ViewId>)> {
        self.pop_frame()
            .map(|frame| (frame.id, frame.root_view, frame.scoped_views))
    }

    /// Active modal id, if a modal is active.
    #[must_use]
    pub fn active_id(&self) -> Option<ModalId> {
        self.frames.last().map(|frame| frame.id)
    }

    /// Root view for the active modal frame, if one was supplied through `show`.
    #[must_use]
    pub fn active_root_view(&self) -> Option<ViewId> {
        self.frames.last().and_then(|frame| frame.root_view)
    }

    /// Attach `view_id` to the active modal scope.
    ///
    /// Returns `true` when a modal frame is active, `false` otherwise.
    pub fn attach_view_to_active(&mut self, view_id: ViewId) -> bool {
        let Some(frame) = self.frames.last_mut() else {
            return false;
        };
        if !frame.scoped_views.contains(&view_id) {
            frame.scoped_views.push(view_id);
        }
        true
    }

    /// Scoped views for the active modal frame, if any.
    #[must_use]
    pub fn active_scoped_views(&self) -> Option<&[ViewId]> {
        self.frames
            .last()
            .map(|frame| frame.scoped_views.as_slice())
    }

    /// Number of active modal frames.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.frames.len()
    }

    /// Remove all active modal frames.
    pub fn clear(&mut self) {
        self.frames.clear();
    }

    /// True when no modal frame is active.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    fn pop_frame(&mut self) -> Option<ModalFrame> {
        self.frames.pop()
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
    fn leave_with_scoped_views_returns_popped_scope() {
        let mut modals = ModalStack::default();
        modals.enter(ModalId(10));
        assert!(modals.attach_view_to_active(ViewId::from_raw(1)));
        assert!(modals.attach_view_to_active(ViewId::from_raw(2)));

        assert_eq!(
            modals.leave_with_scoped_views(),
            Some((ModalId(10), vec![ViewId::from_raw(1), ViewId::from_raw(2)]))
        );
        assert!(modals.is_empty());
    }

    #[test]
    fn show_tracks_root_view_for_active_modal() {
        let mut modals = ModalStack::default();

        modals.show(ModalId(10), ViewId::from_raw(7));

        assert_eq!(modals.active_id(), Some(ModalId(10)));
        assert_eq!(modals.active_root_view(), Some(ViewId::from_raw(7)));
    }

    #[test]
    fn leave_with_scope_info_returns_root_and_manual_scope() {
        let mut modals = ModalStack::default();
        modals.show(ModalId(10), ViewId::from_raw(7));
        assert!(modals.attach_view_to_active(ViewId::from_raw(1)));

        assert_eq!(
            modals.leave_with_scope_info(),
            Some((
                ModalId(10),
                Some(ViewId::from_raw(7)),
                vec![ViewId::from_raw(1)]
            ))
        );
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

    #[test]
    fn attach_view_to_active_modal_tracks_scope() {
        let mut modals = ModalStack::default();
        modals.enter(ModalId(10));

        assert!(modals.attach_view_to_active(ViewId::from_raw(1)));
        assert!(modals.attach_view_to_active(ViewId::from_raw(2)));
        assert_eq!(
            modals.active_scoped_views(),
            Some(&[ViewId::from_raw(1), ViewId::from_raw(2)][..])
        );
    }

    #[test]
    fn attach_view_to_active_modal_deduplicates() {
        let mut modals = ModalStack::default();
        modals.enter(ModalId(10));

        assert!(modals.attach_view_to_active(ViewId::from_raw(1)));
        assert!(modals.attach_view_to_active(ViewId::from_raw(1)));
        assert_eq!(
            modals.active_scoped_views().map(|views| views.len()),
            Some(1)
        );
    }

    #[test]
    fn attach_view_without_active_modal_returns_false() {
        let mut modals = ModalStack::default();
        assert!(!modals.attach_view_to_active(ViewId::from_raw(1)));
    }
}

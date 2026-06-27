//! Modal interaction context: saved return focus, default/cancel actions, and modal results.
//!
//! These extend the modal stack so a closing modal can resolve a result and restore the exact
//! window/focus state that was active before it opened. This is the retained-side foundation for
//! the dialog Accept/Cancel and focus-restoration behavior described in the TUI review (C4).
//!
//! Review: `docs/future/tui/completed.md`
//! Spec: `docs/pascal/std/tui/app/README.md`

use crate::{CommandId, ViewId};

use super::{ModalId, ModalStack};

/// Outcome resolved by a modal dialog before it closes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalResult {
    /// The modal's default control confirmed the dialog.
    Accept,
    /// The modal was cancelled.
    Cancel,
    /// An application-defined result command.
    Command(i64),
}

/// Full context returned when a modal frame closes.
///
/// The host uses this to unregister owned roots, restore the previously active window and focus,
/// and deliver the resolved [`ModalResult`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalClose {
    /// Closed modal id.
    pub id: ModalId,
    /// Root view bound to the modal, if any.
    pub root_view: Option<ViewId>,
    /// Whether the host owns and should unregister `root_view`.
    pub owns_root_view: bool,
    /// Manually attached scope views.
    pub scoped_views: Vec<ViewId>,
    /// Window root that was active before the modal opened.
    pub previous_active_root: Option<ViewId>,
    /// Focused leaf that was active before the modal opened.
    pub previous_focus: Option<ViewId>,
    /// Resolved modal result, if one was set before closing.
    pub result: Option<ModalResult>,
}

impl ModalStack {
    /// Record the window/focus state to restore when the active modal closes.
    ///
    /// Returns `false` when no modal frame is active.
    pub fn set_return_context(
        &mut self,
        previous_active_root: Option<ViewId>,
        previous_focus: Option<ViewId>,
    ) -> bool {
        let Some(frame) = self.frames.last_mut() else {
            return false;
        };
        frame.previous_active_root = previous_active_root;
        frame.previous_focus = previous_focus;
        true
    }

    /// Window root that was active before the active modal opened.
    #[must_use]
    pub fn previous_active_root(&self) -> Option<ViewId> {
        self.frames
            .last()
            .and_then(|frame| frame.previous_active_root)
    }

    /// Focused leaf that was active before the active modal opened.
    #[must_use]
    pub fn previous_focus(&self) -> Option<ViewId> {
        self.frames.last().and_then(|frame| frame.previous_focus)
    }

    /// Bind the active modal's default (Enter) action command.
    ///
    /// Returns `false` when no modal frame is active.
    pub fn set_default_action(&mut self, command_id: CommandId) -> bool {
        let Some(frame) = self.frames.last_mut() else {
            return false;
        };
        frame.default_action = Some(command_id);
        true
    }

    /// Bind the active modal's cancel (Escape) action command.
    ///
    /// Returns `false` when no modal frame is active.
    pub fn set_cancel_action(&mut self, command_id: CommandId) -> bool {
        let Some(frame) = self.frames.last_mut() else {
            return false;
        };
        frame.cancel_action = Some(command_id);
        true
    }

    /// Default (Enter) action command for the active modal, if bound.
    #[must_use]
    pub fn default_action(&self) -> Option<CommandId> {
        self.frames.last().and_then(|frame| frame.default_action)
    }

    /// Cancel (Escape) action command for the active modal, if bound.
    #[must_use]
    pub fn cancel_action(&self) -> Option<CommandId> {
        self.frames.last().and_then(|frame| frame.cancel_action)
    }

    /// Set the resolved result for the active modal.
    ///
    /// Returns `false` when no modal frame is active.
    pub fn set_result(&mut self, result: ModalResult) -> bool {
        let Some(frame) = self.frames.last_mut() else {
            return false;
        };
        frame.result = Some(result);
        true
    }

    /// Resolved result for the active modal, if one was set.
    #[must_use]
    pub fn active_result(&self) -> Option<ModalResult> {
        self.frames.last().and_then(|frame| frame.result)
    }

    /// Pop the active modal and return its full close context.
    #[must_use]
    pub fn leave_with_context(&mut self) -> Option<ModalClose> {
        self.pop_frame().map(|frame| ModalClose {
            id: frame.id,
            root_view: frame.root_view,
            owns_root_view: frame.owns_root_view,
            scoped_views: frame.scoped_views,
            previous_active_root: frame.previous_active_root,
            previous_focus: frame.previous_focus,
            result: frame.result,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_return_context_records_previous_window_and_focus() {
        let mut modals = ModalStack::default();
        modals.show_dialog(ModalId(10), ViewId::from_raw(7));

        assert!(modals.set_return_context(Some(ViewId::from_raw(3)), Some(ViewId::from_raw(4))));
        assert_eq!(modals.previous_active_root(), Some(ViewId::from_raw(3)));
        assert_eq!(modals.previous_focus(), Some(ViewId::from_raw(4)));
    }

    #[test]
    fn set_return_context_without_active_modal_returns_false() {
        let mut modals = ModalStack::default();
        assert!(!modals.set_return_context(Some(ViewId::from_raw(3)), None));
    }

    #[test]
    fn default_and_cancel_actions_track_active_frame() {
        let mut modals = ModalStack::default();
        modals.show_dialog(ModalId(10), ViewId::from_raw(7));

        assert!(modals.set_default_action(CommandId(1)));
        assert!(modals.set_cancel_action(CommandId(2)));
        assert_eq!(modals.default_action(), Some(CommandId(1)));
        assert_eq!(modals.cancel_action(), Some(CommandId(2)));

        // Actions are per-frame: a nested modal starts without inherited actions.
        modals.enter(ModalId(20));
        assert_eq!(modals.default_action(), None);
        assert_eq!(modals.cancel_action(), None);
    }

    #[test]
    fn set_result_records_active_frame_outcome() {
        let mut modals = ModalStack::default();
        modals.show_dialog(ModalId(10), ViewId::from_raw(7));

        assert_eq!(modals.active_result(), None);
        assert!(modals.set_result(ModalResult::Accept));
        assert_eq!(modals.active_result(), Some(ModalResult::Accept));
    }

    #[test]
    fn leave_with_context_returns_saved_state_and_result() {
        let mut modals = ModalStack::default();
        modals.show_dialog(ModalId(10), ViewId::from_raw(7));
        assert!(modals.attach_view_to_active(ViewId::from_raw(8)));
        assert!(modals.set_return_context(Some(ViewId::from_raw(3)), Some(ViewId::from_raw(4))));
        assert!(modals.set_result(ModalResult::Command(99)));

        assert_eq!(
            modals.leave_with_context(),
            Some(ModalClose {
                id: ModalId(10),
                root_view: Some(ViewId::from_raw(7)),
                owns_root_view: true,
                scoped_views: vec![ViewId::from_raw(8)],
                previous_active_root: Some(ViewId::from_raw(3)),
                previous_focus: Some(ViewId::from_raw(4)),
                result: Some(ModalResult::Command(99)),
            })
        );
    }

    #[test]
    fn nested_modals_restore_their_own_return_context() {
        let mut modals = ModalStack::default();
        modals.show_dialog(ModalId(10), ViewId::from_raw(1));
        assert!(modals.set_return_context(None, Some(ViewId::from_raw(2))));

        modals.show_dialog(ModalId(20), ViewId::from_raw(3));
        assert!(modals.set_return_context(Some(ViewId::from_raw(1)), Some(ViewId::from_raw(4))));

        let inner = modals.leave_with_context().expect("inner modal");
        assert_eq!(inner.id, ModalId(20));
        assert_eq!(inner.previous_focus, Some(ViewId::from_raw(4)));

        // The outer modal's saved context is intact after the inner one closes.
        assert_eq!(modals.previous_focus(), Some(ViewId::from_raw(2)));
        let outer = modals.leave_with_context().expect("outer modal");
        assert_eq!(outer.id, ModalId(10));
        assert_eq!(outer.previous_focus, Some(ViewId::from_raw(2)));
    }

    #[test]
    fn remove_view_references_clears_saved_context_views() {
        let mut modals = ModalStack::default();
        modals.show_dialog(ModalId(10), ViewId::from_raw(7));
        assert!(modals.set_return_context(Some(ViewId::from_raw(3)), Some(ViewId::from_raw(4))));

        modals.remove_view_references(&[ViewId::from_raw(3), ViewId::from_raw(4)]);

        assert_eq!(modals.previous_active_root(), None);
        assert_eq!(modals.previous_focus(), None);
    }
}

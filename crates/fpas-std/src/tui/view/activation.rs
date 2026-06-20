//! Active-window management: root activation and click-to-front z-order.
//!
//! These primitives are the desktop/window-manager foundation for the framed window and dialog
//! work tracked in [`docs/future/windows-dialogs/README.md`] and the review's Phase 2.
//!
//! Spec: `docs/pascal/std/tui/app/README.md`

use super::{ViewId, ViewRegistry};

/// Result of activating a window root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootActivation {
    /// Root view that was activated.
    pub root: ViewId,
    /// Whether the root moved to the front of the root z-order.
    pub raised: bool,
    /// Whether keyboard focus moved into the activated root.
    pub focus_changed: bool,
    /// Whether a view was focused before this activation.
    pub had_previous_focus: bool,
}

impl ViewRegistry {
    /// Return the root ancestor of `id`, or `id` itself when it is already a root.
    ///
    /// Returns `None` for unknown view ids.
    #[must_use]
    pub fn root_of(&self, id: ViewId) -> Option<ViewId> {
        self.entry(id)?;
        let mut current = id;
        while let Some(parent) = self.parent(current) {
            current = parent;
        }
        Some(current)
    }

    /// Return the active window root: the root ancestor of the focused leaf.
    ///
    /// Returns `None` when no view is focused.
    #[must_use]
    pub fn active_root(&self) -> Option<ViewId> {
        self.focused_id().and_then(|leaf| self.root_of(leaf))
    }

    /// Activate the window root containing `id`.
    ///
    /// Raises that root to the front of the root z-order (click-to-front) and moves keyboard focus
    /// into its subtree: focus already inside the root is preserved, otherwise the first eligible
    /// Tab stop in the subtree is focused. When the root has no eligible Tab stop, the root is still
    /// raised and the previous focus is left unchanged. Returns `None` for unknown view ids.
    pub fn activate_root(&mut self, id: ViewId) -> Option<RootActivation> {
        let root = self.root_of(id)?;
        let raised = self.raise_root_to_front(root);

        let subtree = self.subtree_ids(root);
        let had_previous_focus = self.focused_id().is_some();
        let focus_changed = if self
            .focused_id()
            .is_some_and(|leaf| subtree.contains(&leaf))
        {
            false
        } else {
            self.focus_first_in_scope(&subtree).0
        };

        Some(RootActivation {
            root,
            raised,
            focus_changed,
            had_previous_focus,
        })
    }

    fn raise_root_to_front(&mut self, root: ViewId) -> bool {
        let Some(position) = self.roots.iter().position(|id| *id == root) else {
            return false;
        };
        if position + 1 == self.roots.len() {
            return false;
        }
        let id = self.roots.remove(position);
        self.roots.push(id);
        true
    }
}

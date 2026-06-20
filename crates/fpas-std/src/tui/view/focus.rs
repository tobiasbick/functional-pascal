//! Focus traversal derived from retained tree order and view state.

use super::{ViewId, ViewRegistry};

impl ViewRegistry {
    /// Marks a registered view selectable and includes it in Tab traversal.
    ///
    /// This is the compatibility adapter for the current `HostPushChildView` bridge. New host
    /// code should set [`ViewOptions`](super::ViewOptions) directly.
    pub fn push_child(&mut self, id: ViewId) -> bool {
        let Some(mut options) = self.options(id) else {
            return false;
        };
        options.selectable = true;
        options.tab_stop = true;
        self.set_options(id, options)
    }

    /// Removes a view from selectable and Tab traversal state.
    pub fn remove_child(&mut self, id: ViewId) {
        let candidates = self.focus_candidates(None, true);
        let previous = candidates.iter().position(|candidate| *candidate == id);
        let was_focused = self.focused == Some(id);
        let Some(mut options) = self.options(id) else {
            return;
        };
        options.selectable = false;
        options.tab_stop = false;
        let _ = self.set_options(id, options);

        if was_focused {
            let remaining = self.focus_candidates(None, true);
            self.focused = previous
                .and_then(|position| position.checked_sub(1))
                .and_then(|position| remaining.get(position).copied());
            self.rebuild_focus_path();
        }
    }

    /// Return the currently focused leaf view id, if one exists.
    #[must_use]
    pub fn focused_id(&self) -> Option<ViewId> {
        self.focused
    }

    /// Return `true` when at least one enabled, visible Tab stop exists.
    #[must_use]
    pub fn has_focusable_children(&self) -> bool {
        !self.focus_candidates(None, true).is_empty()
    }

    /// Focus an enabled, visible selectable view.
    ///
    /// Returns `(changed, had_previous)`.
    pub fn focus_view(&mut self, id: ViewId) -> (bool, bool) {
        if !self.is_focus_candidate(id, false) || self.focused == Some(id) {
            return (false, self.focused.is_some());
        }
        let had_previous = self.focused.is_some();
        self.focused = Some(id);
        self.rebuild_focus_path();
        (true, had_previous)
    }

    /// Focus the first eligible Tab stop that belongs to `scope`.
    pub fn focus_first_in_scope(&mut self, scope: &[ViewId]) -> (bool, bool) {
        let Some(target) = self.focus_candidates(Some(scope), true).first().copied() else {
            return (false, false);
        };
        self.focus_view(target)
    }

    /// Advance focus forward through eligible Tab stops.
    pub fn focus_next(&mut self) -> (bool, bool) {
        self.focus_step(None, true)
    }

    /// Advance focus forward within `scope`.
    pub fn focus_next_in_scope(&mut self, scope: &[ViewId]) -> (bool, bool) {
        self.focus_step(Some(scope), true)
    }

    /// Retreat focus backward through eligible Tab stops.
    pub fn focus_prev(&mut self) -> (bool, bool) {
        self.focus_step(None, false)
    }

    /// Retreat focus backward within `scope`.
    pub fn focus_prev_in_scope(&mut self, scope: &[ViewId]) -> (bool, bool) {
        self.focus_step(Some(scope), false)
    }

    pub(super) fn ensure_valid_focus(&mut self) {
        if self
            .focused
            .is_some_and(|id| !self.is_focus_candidate(id, false))
        {
            self.focused = None;
            self.rebuild_focus_path();
        }
    }

    fn focus_step(&mut self, scope: Option<&[ViewId]>, forward: bool) -> (bool, bool) {
        let candidates = self.focus_candidates(scope, true);
        if candidates.is_empty() {
            return (false, false);
        }

        let had_previous = self.focused.is_some();
        let target = match self
            .focused
            .and_then(|focused| candidates.iter().position(|id| *id == focused))
        {
            Some(position) if forward => candidates[(position + 1) % candidates.len()],
            Some(position) => candidates[(position + candidates.len() - 1) % candidates.len()],
            None if forward => candidates[0],
            None => candidates[candidates.len() - 1],
        };
        if self.focused == Some(target) {
            return (false, had_previous);
        }

        self.focused = Some(target);
        self.rebuild_focus_path();
        (true, had_previous)
    }

    fn focus_candidates(&self, scope: Option<&[ViewId]>, require_tab_stop: bool) -> Vec<ViewId> {
        self.paint_order()
            .into_iter()
            .filter(|id| scope.is_none_or(|ids| ids.contains(id)))
            .filter(|id| self.is_focus_candidate(*id, require_tab_stop))
            .collect()
    }

    fn is_focus_candidate(&self, id: ViewId, require_tab_stop: bool) -> bool {
        self.resolved(id).is_some_and(|view| {
            view.state.exposed
                && view.state.enabled
                && view.options.selectable
                && (!require_tab_stop || view.options.tab_stop)
        })
    }

    fn rebuild_focus_path(&mut self) {
        for entry in &mut self.views {
            entry.current_child = None;
        }

        let Some(focused) = self.focused else {
            return;
        };
        let path = self.ancestors_inclusive(focused);
        for pair in path.windows(2) {
            let child = pair[0];
            let parent = pair[1];
            if let Some(entry) = self.entry_mut(parent) {
                entry.current_child = Some(child);
            }
        }
    }
}

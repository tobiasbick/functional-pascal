//! Rust-internal view registry for the TUI application framework (Phase 7).
//!
//! View handles are opaque identifiers maintained entirely by the host; FPAS has no
//! surface for them yet.  The focus chain tracks an ordered sequence of focusable
//! children; Tab / Shift+Tab traversal is managed by the VM run loop.
//!
//! Plan: `docs/future/tui-application-framework.md`
//! Spec: `docs/pascal/std/tui-app.md`

/// Opaque handle identifying a host-managed view.
///
/// FPAS has no surface for this yet. Will be exposed as a first-class type in a
/// later Phase 7 step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewId(u32);

impl ViewId {
    /// Construct a view id from its raw host representation.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Raw host representation used when shuttling view ids through FPAS today.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Axis-aligned bounding box for a view (terminal cell coordinates, origin top-left).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewRect {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

#[derive(Debug)]
struct ViewEntry {
    id: ViewId,
    rect: ViewRect,
}

/// Host-side registry for all active views in a TUI session.
///
/// Allocates monotonically increasing [`ViewId`]s. Registration order determines
/// initial paint order (index 0 = bottom / background).
///
/// The *focus chain* is a separate ordered list of [`ViewId`]s — a subset of
/// registered views that participate in Tab / Shift+Tab traversal.  Adding a view
/// to the chain is explicit via [`push_child`][Self::push_child]; removal happens
/// automatically on [`unregister`][Self::unregister] or [`clear`][Self::clear].
#[derive(Debug, Default)]
pub struct ViewRegistry {
    next_id: u32,
    views: Vec<ViewEntry>,
    /// Ordered focus chain (Tab / Shift+Tab traversal order).
    children: Vec<ViewId>,
    /// Index into `children` for the currently focused view, or `None` if no view
    /// in the chain has been focused yet.
    focused: Option<usize>,
}

impl ViewRegistry {
    /// Register a new view covering `rect`. Returns its opaque [`ViewId`].
    pub fn register(&mut self, rect: ViewRect) -> ViewId {
        let id = ViewId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.views.push(ViewEntry { id, rect });
        id
    }

    /// Remove a view by id. No-op if not found.
    ///
    /// If the view is in the focus chain it is also removed there; the focused
    /// index is adjusted to keep the focus valid.
    pub fn unregister(&mut self, id: ViewId) {
        self.views.retain(|v| v.id != id);
        self.remove_from_focus_chain(id);
    }

    /// Bounding rect for a view, or `None` if not registered.
    #[must_use]
    pub fn rect(&self, id: ViewId) -> Option<ViewRect> {
        self.views.iter().find(|v| v.id == id).map(|v| v.rect)
    }

    /// Update the bounding rect for a registered view. No-op if not found.
    pub fn set_rect(&mut self, id: ViewId, rect: ViewRect) {
        if let Some(entry) = self.views.iter_mut().find(|v| v.id == id) {
            entry.rect = rect;
        }
    }

    /// All registered view IDs, in registration order (bottom to top).
    pub fn ids(&self) -> impl Iterator<Item = ViewId> + '_ {
        self.views.iter().map(|v| v.id)
    }

    /// Count of registered views.
    #[must_use]
    pub fn len(&self) -> usize {
        self.views.len()
    }

    /// True when no views are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.views.is_empty()
    }

    /// Remove all registered views (called on `Application.Close`).
    pub fn clear(&mut self) {
        self.views.clear();
        self.children.clear();
        self.focused = None;
    }

    // --- Focus chain ---

    /// Append `id` to the focus chain if it is not already present.
    ///
    /// The view does not need to be registered via [`register`][Self::register] first,
    /// but only registered views have meaningful bounding boxes.
    pub fn push_child(&mut self, id: ViewId) {
        if !self.children.contains(&id) {
            self.children.push(id);
        }
    }

    /// Remove `id` from the focus chain. No-op if not present.
    ///
    /// The focused index is adjusted so that another child (if any) retains focus.
    pub fn remove_child(&mut self, id: ViewId) {
        self.remove_from_focus_chain(id);
    }

    /// `ViewId` of the currently focused child, or `None` if the chain is empty or
    /// no view has been focused yet.
    #[must_use]
    pub fn focused_id(&self) -> Option<ViewId> {
        self.focused.map(|i| self.children[i])
    }

    /// `true` when the focus chain has at least one entry.
    #[must_use]
    pub fn has_focusable_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Advance focus forward (Tab).
    ///
    /// Returns `(changed, had_previous)`:
    /// - `changed`: focus moved or was established.
    /// - `had_previous`: there was a previously focused view before the transition
    ///   (caller should fire `OnDeactivate` for the old view only when `true`).
    ///
    /// Returns `(false, false)` when the chain is empty or has only one already-focused
    /// entry (Tab wraps for chains with two or more entries).
    pub fn focus_next(&mut self) -> (bool, bool) {
        self.focus_step(1)
    }

    /// Advance focus forward (Tab) within `scope`.
    pub fn focus_next_in_scope(&mut self, scope: &[ViewId]) -> (bool, bool) {
        self.focus_step_in_scope(scope, true)
    }

    /// Retreat focus backward (Shift+Tab).
    ///
    /// Same return semantics as [`focus_next`][Self::focus_next].
    pub fn focus_prev(&mut self) -> (bool, bool) {
        self.focus_step(self.children.len().saturating_sub(1))
    }

    /// Retreat focus backward (Shift+Tab) within `scope`.
    pub fn focus_prev_in_scope(&mut self, scope: &[ViewId]) -> (bool, bool) {
        self.focus_step_in_scope(scope, false)
    }

    // --- Private helpers ---

    fn focus_step(&mut self, step: usize) -> (bool, bool) {
        let len = self.children.len();
        match len {
            0 => (false, false),
            1 => {
                if self.focused.is_none() {
                    self.focused = Some(0);
                    (true, false)
                } else {
                    (false, false)
                }
            }
            _ => {
                let had_previous = self.focused.is_some();
                let new_idx = self.focused.map_or(0, |i| (i + step) % len);
                self.focused = Some(new_idx);
                (true, had_previous)
            }
        }
    }

    fn focus_step_in_scope(&mut self, scope: &[ViewId], forward: bool) -> (bool, bool) {
        let scoped_indices: Vec<usize> = self
            .children
            .iter()
            .enumerate()
            .filter_map(|(idx, id)| scope.contains(id).then_some(idx))
            .collect();

        match scoped_indices.len() {
            0 => (false, false),
            1 => {
                let target = scoped_indices[0];
                if self.focused == Some(target) {
                    (false, false)
                } else {
                    let had_previous = self.focused.is_some();
                    self.focused = Some(target);
                    (true, had_previous)
                }
            }
            len => {
                let had_previous = self.focused.is_some();
                let target = match self
                    .focused
                    .and_then(|focused| scoped_indices.iter().position(|&idx| idx == focused))
                {
                    Some(pos) => {
                        if forward {
                            scoped_indices[(pos + 1) % len]
                        } else {
                            scoped_indices[(pos + len - 1) % len]
                        }
                    }
                    None => {
                        if forward {
                            scoped_indices[0]
                        } else {
                            scoped_indices[len - 1]
                        }
                    }
                };
                if self.focused == Some(target) {
                    (false, false)
                } else {
                    self.focused = Some(target);
                    (true, had_previous)
                }
            }
        }
    }

    fn remove_from_focus_chain(&mut self, id: ViewId) {
        let Some(pos) = self.children.iter().position(|&v| v == id) else {
            return;
        };
        self.children.remove(pos);
        self.focused = match self.focused {
            None => None,
            Some(i) if i > pos => Some(i - 1),
            Some(i) if i == pos => {
                if self.children.is_empty() {
                    None
                } else {
                    Some(i.saturating_sub(1))
                }
            }
            Some(i) => Some(i),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: i64, y: i64, w: i64, h: i64) -> ViewRect {
        ViewRect {
            x,
            y,
            width: w,
            height: h,
        }
    }

    #[test]
    fn register_returns_distinct_ids() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 10, 5));
        let b = reg.register(rect(10, 0, 20, 5));
        assert_ne!(a, b);
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn rect_lookup_round_trips() {
        let mut reg = ViewRegistry::default();
        let r = rect(3, 7, 40, 20);
        let id = reg.register(r);
        assert_eq!(reg.rect(id), Some(r));
    }

    #[test]
    fn unregister_removes_view() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 10, 5));
        let b = reg.register(rect(0, 5, 10, 5));
        reg.unregister(a);
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.rect(a), None);
        assert!(reg.rect(b).is_some());
    }

    #[test]
    fn set_rect_updates_bounds() {
        let mut reg = ViewRegistry::default();
        let id = reg.register(rect(0, 0, 10, 5));
        let new_r = rect(1, 2, 80, 24);
        reg.set_rect(id, new_r);
        assert_eq!(reg.rect(id), Some(new_r));
    }

    #[test]
    fn ids_preserves_registration_order() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 1, 1));
        let b = reg.register(rect(1, 0, 1, 1));
        let c = reg.register(rect(2, 0, 1, 1));
        let ids: Vec<_> = reg.ids().collect();
        assert_eq!(ids, vec![a, b, c]);
    }

    #[test]
    fn clear_removes_all_views() {
        let mut reg = ViewRegistry::default();
        reg.register(rect(0, 0, 10, 5));
        reg.register(rect(0, 5, 10, 5));
        reg.clear();
        assert!(reg.is_empty());
    }

    #[test]
    fn unregister_unknown_id_is_noop() {
        let mut reg = ViewRegistry::default();
        let id = reg.register(rect(0, 0, 10, 5));
        reg.unregister(id);
        // second unregister must not panic
        reg.unregister(id);
        assert!(reg.is_empty());
    }

    // --- Focus chain tests ---

    #[test]
    fn push_child_adds_to_focus_chain() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 10, 5));
        let b = reg.register(rect(0, 5, 10, 5));
        reg.push_child(a);
        reg.push_child(b);
        assert!(reg.has_focusable_children());
    }

    #[test]
    fn push_child_deduplicates() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 10, 5));
        reg.push_child(a);
        reg.push_child(a);
        // Only one entry; focus_next on 1-element with no prior focus establishes focus.
        let (changed, had) = reg.focus_next();
        assert!(changed);
        assert!(!had);
        // Second Tab on single-element chain does nothing.
        let (changed2, _) = reg.focus_next();
        assert!(!changed2);
    }

    #[test]
    fn focus_next_empty_chain_returns_no_change() {
        let mut reg = ViewRegistry::default();
        assert_eq!(reg.focus_next(), (false, false));
    }

    #[test]
    fn focus_next_single_establishes_focus() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 10, 5));
        reg.push_child(a);
        assert_eq!(reg.focused_id(), None);
        let (changed, had) = reg.focus_next();
        assert!(changed);
        assert!(!had);
        assert_eq!(reg.focused_id(), Some(a));
    }

    #[test]
    fn focus_next_two_children_wraps() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 10, 5));
        let b = reg.register(rect(0, 5, 10, 5));
        reg.push_child(a);
        reg.push_child(b);

        // First Tab: establish focus on a (index 0).
        let (c1, h1) = reg.focus_next();
        assert!(c1);
        assert!(!h1);
        assert_eq!(reg.focused_id(), Some(a));

        // Second Tab: move to b.
        let (c2, h2) = reg.focus_next();
        assert!(c2);
        assert!(h2);
        assert_eq!(reg.focused_id(), Some(b));

        // Third Tab: wrap back to a.
        let (c3, h3) = reg.focus_next();
        assert!(c3);
        assert!(h3);
        assert_eq!(reg.focused_id(), Some(a));
    }

    #[test]
    fn focus_prev_retreats() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 10, 5));
        let b = reg.register(rect(0, 5, 10, 5));
        let c = reg.register(rect(0, 10, 10, 5));
        reg.push_child(a);
        reg.push_child(b);
        reg.push_child(c);

        // Establish forward focus to b.
        reg.focus_next(); // a
        reg.focus_next(); // b

        // Shift+Tab: back to a.
        let (changed, had) = reg.focus_prev();
        assert!(changed);
        assert!(had);
        assert_eq!(reg.focused_id(), Some(a));
    }

    #[test]
    fn remove_child_adjusts_focus() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 10, 5));
        let b = reg.register(rect(0, 5, 10, 5));
        reg.push_child(a);
        reg.push_child(b);
        // Focus b (index 1).
        reg.focus_next(); // a
        reg.focus_next(); // b
        assert_eq!(reg.focused_id(), Some(b));
        // Remove b — focus should fall back to a (index 0).
        reg.remove_child(b);
        assert_eq!(reg.focused_id(), Some(a));
    }

    #[test]
    fn unregister_also_removes_from_focus_chain() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 10, 5));
        let b = reg.register(rect(0, 5, 10, 5));
        reg.push_child(a);
        reg.push_child(b);
        reg.focus_next(); // a
        reg.focus_next(); // b
        // Unregister b: removed from both view list and focus chain.
        reg.unregister(b);
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.focused_id(), Some(a));
    }

    #[test]
    fn clear_resets_focus_chain() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 10, 5));
        reg.push_child(a);
        reg.focus_next();
        reg.clear();
        assert!(!reg.has_focusable_children());
        assert_eq!(reg.focused_id(), None);
    }

    #[test]
    fn view_id_raw_round_trip() {
        let id = ViewId::from_raw(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn focus_next_in_scope_skips_non_modal_children() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 10, 5));
        let b = reg.register(rect(0, 5, 10, 5));
        let c = reg.register(rect(0, 10, 10, 5));
        reg.push_child(a);
        reg.push_child(b);
        reg.push_child(c);

        reg.focus_next(); // a
        let (changed, had_previous) = reg.focus_next_in_scope(&[b, c]);

        assert!(changed);
        assert!(had_previous);
        assert_eq!(reg.focused_id(), Some(b));
    }

    #[test]
    fn focus_prev_in_scope_establishes_last_scoped_view() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 10, 5));
        let b = reg.register(rect(0, 5, 10, 5));
        let c = reg.register(rect(0, 10, 10, 5));
        reg.push_child(a);
        reg.push_child(b);
        reg.push_child(c);

        let (changed, had_previous) = reg.focus_prev_in_scope(&[b, c]);

        assert!(changed);
        assert!(!had_previous);
        assert_eq!(reg.focused_id(), Some(c));
    }
}

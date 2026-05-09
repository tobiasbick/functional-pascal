//! Rust-internal view registry for the TUI application framework (Phase 7).
//!
//! View handles are opaque identifiers maintained entirely by the host. The registry tracks a
//! small view tree: root views use absolute terminal coordinates, child views use coordinates
//! relative to their parent, and sibling order defines paint and hit-test z-order.
//!
//! Plan: `docs/future/tui-application-framework.md`
//! Spec: `docs/pascal/std/tui-app.md`

/// Opaque handle identifying a host-managed view.
///
/// FPAS still treats this as an integer token on the Pascal side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewId(u32);

impl ViewId {
    /// Construct a view id from its raw host representation.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Return the raw host representation used in the VM bridge today.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Axis-aligned bounding box for a view in terminal-cell coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewRect {
    /// Left edge in terminal cells.
    pub x: i64,
    /// Top edge in terminal cells.
    pub y: i64,
    /// Width in terminal cells.
    pub width: i64,
    /// Height in terminal cells.
    pub height: i64,
}

#[derive(Debug)]
struct ViewEntry {
    id: ViewId,
    local_rect: ViewRect,
    parent: Option<ViewId>,
    children: Vec<ViewId>,
}

/// Host-side registry for all active views in a TUI session.
///
/// Root views use absolute terminal coordinates. Child views use coordinates relative to their
/// parent. Sibling insertion order defines z-order: later siblings paint later and therefore sit
/// on top during hit-testing.
///
/// The focus chain remains a separate ordered list of [`ViewId`]s. Adding a view to that chain is
/// still explicit via [`push_child`][Self::push_child].
#[derive(Debug, Default)]
pub struct ViewRegistry {
    next_id: u32,
    views: Vec<ViewEntry>,
    roots: Vec<ViewId>,
    children: Vec<ViewId>,
    focused: Option<usize>,
}

impl ViewRegistry {
    /// Register a new root view covering `rect` and return its opaque [`ViewId`].
    pub fn register(&mut self, rect: ViewRect) -> ViewId {
        let id = ViewId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.views.push(ViewEntry {
            id,
            local_rect: rect,
            parent: None,
            children: Vec::new(),
        });
        self.roots.push(id);
        id
    }

    /// Remove a view by id.
    ///
    /// Removing a view also removes its full subtree and any focus-chain entries that pointed to
    /// those views. Unknown ids are ignored.
    pub fn unregister(&mut self, id: ViewId) {
        let subtree = self.subtree_ids(id);
        if subtree.is_empty() {
            return;
        }

        let parent = self.entry(id).and_then(|entry| entry.parent);
        self.detach_from_parent_or_roots(id, parent);
        for view_id in &subtree {
            self.remove_from_focus_chain(*view_id);
        }
        self.views.retain(|entry| !subtree.contains(&entry.id));
    }

    /// Return the absolute terminal rectangle for `id`, or `None` when it is not registered.
    #[must_use]
    pub fn rect(&self, id: ViewId) -> Option<ViewRect> {
        self.resolve_rect(id)
    }

    /// Update the rectangle for a registered view.
    ///
    /// Root views interpret `rect` as absolute screen coordinates. Child views interpret `rect` as
    /// coordinates relative to their parent.
    pub fn set_rect(&mut self, id: ViewId, rect: ViewRect) {
        if let Some(entry) = self.entry_mut(id) {
            entry.local_rect = rect;
        }
    }

    /// Re-parent `id` under `parent`.
    ///
    /// The view keeps its current absolute screen rectangle during the re-parenting step. Pass
    /// `None` to detach the view back to the root list. Returns `false` for unknown ids or when
    /// the requested parent would introduce a cycle.
    pub fn set_parent(&mut self, id: ViewId, parent: Option<ViewId>) -> bool {
        if parent == Some(id) || self.entry(id).is_none() {
            return false;
        }

        if let Some(parent_id) = parent {
            if self.entry(parent_id).is_none() || self.would_create_cycle(id, parent_id) {
                return false;
            }
        }

        let current_parent = self.entry(id).and_then(|entry| entry.parent);
        if current_parent == parent {
            return true;
        }

        let absolute_rect = match self.rect(id) {
            Some(rect) => rect,
            None => return false,
        };
        let (parent_x, parent_y) = parent
            .and_then(|parent_id| self.rect(parent_id))
            .map(|rect| (rect.x, rect.y))
            .unwrap_or((0, 0));

        self.detach_from_parent_or_roots(id, current_parent);
        match parent {
            Some(parent_id) => {
                if let Some(entry) = self.entry_mut(parent_id) {
                    entry.children.push(id);
                }
            }
            None => self.roots.push(id),
        }

        if let Some(entry) = self.entry_mut(id) {
            entry.parent = parent;
            entry.local_rect = ViewRect {
                x: absolute_rect.x.saturating_sub(parent_x),
                y: absolute_rect.y.saturating_sub(parent_y),
                width: absolute_rect.width,
                height: absolute_rect.height,
            };
        }

        true
    }

    /// Raise `id` to the front of its sibling list.
    ///
    /// Returns `false` when the view is unknown.
    pub fn raise(&mut self, id: ViewId) -> bool {
        let Some(parent) = self.entry(id).map(|entry| entry.parent) else {
            return false;
        };
        self.raise_in_list(id, parent)
    }

    /// Return all registered view ids in registration order.
    pub fn ids(&self) -> impl Iterator<Item = ViewId> + '_ {
        self.views.iter().map(|entry| entry.id)
    }

    /// Return the subtree rooted at `root` in paint order.
    #[must_use]
    pub fn subtree_ids(&self, root: ViewId) -> Vec<ViewId> {
        let mut ids = Vec::new();
        if self.entry(root).is_some() {
            self.collect_subtree(root, &mut ids);
        }
        ids
    }

    /// Return the full paint order from back to front.
    #[must_use]
    pub fn paint_order(&self) -> Vec<ViewId> {
        let mut ids = Vec::new();
        for root in &self.roots {
            self.collect_subtree(*root, &mut ids);
        }
        ids
    }

    /// Return the topmost view at terminal position `(x, y)`.
    ///
    /// When `scope` is present, hit-testing is restricted to that subset of view ids.
    #[must_use]
    pub fn topmost_view_at(&self, x: i64, y: i64, scope: Option<&[ViewId]>) -> Option<ViewId> {
        self.paint_order().into_iter().rev().find(|view_id| {
            scope.map_or(true, |scope_ids| scope_ids.contains(view_id))
                && self
                    .rect(*view_id)
                    .is_some_and(|rect| rect_contains_point(rect, x, y))
        })
    }

    /// Return the number of registered views.
    #[must_use]
    pub fn len(&self) -> usize {
        self.views.len()
    }

    /// Return `true` when no views are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.views.is_empty()
    }

    /// Remove all registered views and all focus-chain state.
    pub fn clear(&mut self) {
        self.views.clear();
        self.roots.clear();
        self.children.clear();
        self.focused = None;
    }

    /// Append `id` to the focus chain if it is not already present.
    pub fn push_child(&mut self, id: ViewId) {
        if !self.children.contains(&id) {
            self.children.push(id);
        }
    }

    /// Remove `id` from the focus chain.
    pub fn remove_child(&mut self, id: ViewId) {
        self.remove_from_focus_chain(id);
    }

    /// Return the currently focused child view id, if one exists.
    #[must_use]
    pub fn focused_id(&self) -> Option<ViewId> {
        self.focused.map(|index| self.children[index])
    }

    /// Return `true` when the focus chain has at least one entry.
    #[must_use]
    pub fn has_focusable_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Focus the first child that belongs to `scope`.
    ///
    /// Returns `(changed, had_previous)`.
    pub fn focus_first_in_scope(&mut self, scope: &[ViewId]) -> (bool, bool) {
        let Some(target) = self.children.iter().position(|id| scope.contains(id)) else {
            return (false, false);
        };
        if self.focused == Some(target) {
            return (false, false);
        }

        let had_previous = self.focused.is_some();
        self.focused = Some(target);
        (true, had_previous)
    }

    /// Advance focus forward through the full focus chain.
    pub fn focus_next(&mut self) -> (bool, bool) {
        self.focus_step(1)
    }

    /// Advance focus forward within `scope`.
    pub fn focus_next_in_scope(&mut self, scope: &[ViewId]) -> (bool, bool) {
        self.focus_step_in_scope(scope, true)
    }

    /// Retreat focus backward through the full focus chain.
    pub fn focus_prev(&mut self) -> (bool, bool) {
        self.focus_step(self.children.len().saturating_sub(1))
    }

    /// Retreat focus backward within `scope`.
    pub fn focus_prev_in_scope(&mut self, scope: &[ViewId]) -> (bool, bool) {
        self.focus_step_in_scope(scope, false)
    }

    fn focus_step(&mut self, step: usize) -> (bool, bool) {
        match self.children.len() {
            0 => (false, false),
            1 => {
                if self.focused.is_none() {
                    self.focused = Some(0);
                    (true, false)
                } else {
                    (false, false)
                }
            }
            len => {
                let had_previous = self.focused.is_some();
                let new_index = self.focused.map_or(0, |index| (index + step) % len);
                self.focused = Some(new_index);
                (true, had_previous)
            }
        }
    }

    fn focus_step_in_scope(&mut self, scope: &[ViewId], forward: bool) -> (bool, bool) {
        let scoped_indices: Vec<usize> = self
            .children
            .iter()
            .enumerate()
            .filter_map(|(index, id)| scope.contains(id).then_some(index))
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
                    .and_then(|focused| scoped_indices.iter().position(|&index| index == focused))
                {
                    Some(position) => {
                        if forward {
                            scoped_indices[(position + 1) % len]
                        } else {
                            scoped_indices[(position + len - 1) % len]
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

    fn entry(&self, id: ViewId) -> Option<&ViewEntry> {
        self.views.iter().find(|entry| entry.id == id)
    }

    fn entry_mut(&mut self, id: ViewId) -> Option<&mut ViewEntry> {
        self.views.iter_mut().find(|entry| entry.id == id)
    }

    fn resolve_rect(&self, id: ViewId) -> Option<ViewRect> {
        let entry = self.entry(id)?;
        let mut rect = entry.local_rect;
        if let Some(parent) = entry.parent {
            let parent_rect = self.resolve_rect(parent)?;
            rect.x = rect.x.saturating_add(parent_rect.x);
            rect.y = rect.y.saturating_add(parent_rect.y);
        }
        Some(rect)
    }

    fn collect_subtree(&self, id: ViewId, ids: &mut Vec<ViewId>) {
        ids.push(id);
        let Some(entry) = self.entry(id) else {
            return;
        };
        for child in &entry.children {
            self.collect_subtree(*child, ids);
        }
    }

    fn would_create_cycle(&self, child: ViewId, parent: ViewId) -> bool {
        let mut current = Some(parent);
        while let Some(view_id) = current {
            if view_id == child {
                return true;
            }
            current = self.entry(view_id).and_then(|entry| entry.parent);
        }
        false
    }

    fn detach_from_parent_or_roots(&mut self, id: ViewId, parent: Option<ViewId>) {
        match parent {
            Some(parent_id) => {
                if let Some(entry) = self.entry_mut(parent_id) {
                    entry.children.retain(|child| *child != id);
                }
            }
            None => self.roots.retain(|root| *root != id),
        }
    }

    fn raise_in_list(&mut self, id: ViewId, parent: Option<ViewId>) -> bool {
        let entries = match parent {
            Some(parent_id) => match self.entry_mut(parent_id) {
                Some(entry) => &mut entry.children,
                None => return false,
            },
            None => &mut self.roots,
        };

        let Some(position) = entries.iter().position(|view_id| *view_id == id) else {
            return false;
        };
        let view_id = entries.remove(position);
        entries.push(view_id);
        true
    }

    fn remove_from_focus_chain(&mut self, id: ViewId) {
        let Some(position) = self.children.iter().position(|view_id| *view_id == id) else {
            return;
        };

        self.children.remove(position);
        self.focused = match self.focused {
            None => None,
            Some(index) if index > position => Some(index - 1),
            Some(index) if index == position => {
                if self.children.is_empty() {
                    None
                } else {
                    Some(index.saturating_sub(1))
                }
            }
            Some(index) => Some(index),
        };
    }
}

fn rect_contains_point(rect: ViewRect, x: i64, y: i64) -> bool {
    let max_x = rect.x.saturating_add(rect.width.max(0));
    let max_y = rect.y.saturating_add(rect.height.max(0));
    x >= rect.x && y >= rect.y && x < max_x && y < max_y
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: i64, y: i64, width: i64, height: i64) -> ViewRect {
        ViewRect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn register_returns_distinct_ids() {
        let mut registry = ViewRegistry::default();
        let a = registry.register(rect(0, 0, 10, 5));
        let b = registry.register(rect(10, 0, 20, 5));

        assert_ne!(a, b);
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn child_rect_tracks_parent_layout() {
        let mut registry = ViewRegistry::default();
        let parent = registry.register(rect(10, 5, 20, 10));
        let child = registry.register(rect(2, 3, 4, 2));

        assert!(registry.set_parent(child, Some(parent)));
        registry.set_rect(child, rect(1, 1, 4, 2));
        assert_eq!(registry.rect(child), Some(rect(11, 6, 4, 2)));

        registry.set_rect(parent, rect(20, 10, 20, 10));
        assert_eq!(registry.rect(child), Some(rect(21, 11, 4, 2)));
    }

    #[test]
    fn reparent_preserves_absolute_rect() {
        let mut registry = ViewRegistry::default();
        let first_parent = registry.register(rect(10, 5, 20, 10));
        let second_parent = registry.register(rect(40, 20, 20, 10));
        let child = registry.register(rect(14, 9, 4, 2));

        assert!(registry.set_parent(child, Some(first_parent)));
        assert_eq!(registry.rect(child), Some(rect(14, 9, 4, 2)));

        assert!(registry.set_parent(child, Some(second_parent)));
        assert_eq!(registry.rect(child), Some(rect(14, 9, 4, 2)));
    }

    #[test]
    fn reparent_rejects_cycles() {
        let mut registry = ViewRegistry::default();
        let root = registry.register(rect(0, 0, 10, 10));
        let child = registry.register(rect(1, 1, 4, 4));
        let grandchild = registry.register(rect(1, 1, 2, 2));

        assert!(registry.set_parent(child, Some(root)));
        assert!(registry.set_parent(grandchild, Some(child)));
        assert!(!registry.set_parent(root, Some(grandchild)));
    }

    #[test]
    fn unregister_removes_subtree() {
        let mut registry = ViewRegistry::default();
        let parent = registry.register(rect(0, 0, 10, 5));
        let child = registry.register(rect(1, 1, 4, 2));
        registry.push_child(parent);
        registry.push_child(child);
        assert!(registry.set_parent(child, Some(parent)));

        registry.unregister(parent);

        assert!(registry.is_empty());
        assert_eq!(registry.focused_id(), None);
        assert_eq!(registry.rect(child), None);
    }

    #[test]
    fn paint_order_follows_tree_and_raise() {
        let mut registry = ViewRegistry::default();
        let background = registry.register(rect(0, 0, 80, 25));
        let window = registry.register(rect(10, 5, 20, 10));
        let button = registry.register(rect(1, 1, 6, 1));

        assert!(registry.set_parent(button, Some(window)));
        assert_eq!(registry.paint_order(), vec![background, window, button]);

        assert!(registry.raise(background));
        assert_eq!(registry.paint_order(), vec![window, button, background]);
    }

    #[test]
    fn topmost_view_at_uses_current_z_order() {
        let mut registry = ViewRegistry::default();
        let back = registry.register(rect(0, 0, 10, 10));
        let front = registry.register(rect(0, 0, 10, 10));

        assert_eq!(registry.topmost_view_at(2, 2, None), Some(front));
        assert!(registry.raise(back));
        assert_eq!(registry.topmost_view_at(2, 2, None), Some(back));
    }

    #[test]
    fn topmost_view_at_can_be_scoped_to_subtree() {
        let mut registry = ViewRegistry::default();
        let root = registry.register(rect(0, 0, 30, 10));
        let sibling = registry.register(rect(0, 0, 30, 10));
        let child = registry.register(rect(1, 1, 10, 3));
        assert!(registry.set_parent(child, Some(root)));

        assert_eq!(registry.topmost_view_at(2, 2, None), Some(sibling));
        assert_eq!(
            registry.topmost_view_at(2, 2, Some(&registry.subtree_ids(root))),
            Some(child)
        );
    }

    #[test]
    fn focus_first_in_scope_targets_first_matching_focus_child() {
        let mut registry = ViewRegistry::default();
        let a = registry.register(rect(0, 0, 1, 1));
        let b = registry.register(rect(1, 0, 1, 1));
        let c = registry.register(rect(2, 0, 1, 1));
        registry.push_child(a);
        registry.push_child(b);
        registry.push_child(c);

        let (changed, had_previous) = registry.focus_first_in_scope(&[b, c]);
        assert!(changed);
        assert!(!had_previous);
        assert_eq!(registry.focused_id(), Some(b));
    }

    #[test]
    fn focus_next_two_children_wraps() {
        let mut registry = ViewRegistry::default();
        let a = registry.register(rect(0, 0, 10, 5));
        let b = registry.register(rect(0, 5, 10, 5));
        registry.push_child(a);
        registry.push_child(b);

        let (changed_a, had_a) = registry.focus_next();
        assert!(changed_a);
        assert!(!had_a);
        assert_eq!(registry.focused_id(), Some(a));

        let (changed_b, had_b) = registry.focus_next();
        assert!(changed_b);
        assert!(had_b);
        assert_eq!(registry.focused_id(), Some(b));

        let (changed_wrap, had_wrap) = registry.focus_next();
        assert!(changed_wrap);
        assert!(had_wrap);
        assert_eq!(registry.focused_id(), Some(a));
    }

    #[test]
    fn remove_child_adjusts_focus() {
        let mut registry = ViewRegistry::default();
        let a = registry.register(rect(0, 0, 1, 1));
        let b = registry.register(rect(1, 0, 1, 1));
        let c = registry.register(rect(2, 0, 1, 1));
        registry.push_child(a);
        registry.push_child(b);
        registry.push_child(c);

        let _ = registry.focus_next();
        let _ = registry.focus_next();
        assert_eq!(registry.focused_id(), Some(b));

        registry.remove_child(b);
        assert_eq!(registry.focused_id(), Some(a));
    }

    #[test]
    fn scoped_focus_wraps_inside_scope_only() {
        let mut registry = ViewRegistry::default();
        let a = registry.register(rect(0, 0, 1, 1));
        let b = registry.register(rect(1, 0, 1, 1));
        let c = registry.register(rect(2, 0, 1, 1));
        registry.push_child(a);
        registry.push_child(b);
        registry.push_child(c);

        let (changed_first, had_previous_first) = registry.focus_first_in_scope(&[b, c]);
        assert!(changed_first);
        assert!(!had_previous_first);
        assert_eq!(registry.focused_id(), Some(b));

        let (changed_next, had_previous_next) = registry.focus_next_in_scope(&[b, c]);
        assert!(changed_next);
        assert!(had_previous_next);
        assert_eq!(registry.focused_id(), Some(c));

        let (changed_wrap, had_previous_wrap) = registry.focus_next_in_scope(&[b, c]);
        assert!(changed_wrap);
        assert!(had_previous_wrap);
        assert_eq!(registry.focused_id(), Some(b));
    }
}

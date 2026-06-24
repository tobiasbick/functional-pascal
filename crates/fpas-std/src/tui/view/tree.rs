use super::{ViewEntry, ViewId, ViewLayout, ViewOptions, ViewRect, ViewRegistry, ViewState};

impl ViewRegistry {
    /// Register a new root view covering `rect` and return its opaque [`ViewId`].
    pub fn register(&mut self, rect: ViewRect) -> ViewId {
        self.register_with_options(rect, ViewOptions::default())
    }

    /// Register a root view with explicit behavior options.
    pub fn register_with_options(&mut self, rect: ViewRect, options: ViewOptions) -> ViewId {
        let id = self.allocate_id();
        self.views.push(ViewEntry {
            id,
            local_rect: rect,
            parent: None,
            children: Vec::new(),
            current_child: None,
            state: ViewState::default(),
            options,
            layout: ViewLayout::default(),
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
        for view_id in &subtree {
            self.remove_child(*view_id);
            if self.pointer_capture == Some(*view_id) {
                self.pointer_capture = None;
            }
        }
        self.clear_frame_roots_in_subtree(&subtree);
        self.detach_from_parent_or_roots(id, parent);
        self.views.retain(|entry| !subtree.contains(&entry.id));
        for entry in &mut self.views {
            if entry.current_child.is_some_and(|id| subtree.contains(&id)) {
                entry.current_child = None;
            }
        }
    }

    /// Return the absolute terminal rectangle for `id`, or `None` when it is not registered.
    #[must_use]
    pub fn rect(&self, id: ViewId) -> Option<ViewRect> {
        self.resolved(id).map(|view| view.rect)
    }

    /// Update the rectangle for a registered view.
    ///
    /// Root views interpret `rect` as absolute screen coordinates. Child views interpret `rect` as
    /// coordinates relative to their parent (frame-root children use the inner viewport origin).
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

        if let Some(parent_id) = parent
            && (self.entry(parent_id).is_none() || self.would_create_cycle(id, parent_id))
        {
            return false;
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
            .and_then(|parent_id| self.parent_local_origin(parent_id))
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

    /// Return root view ids in root-list order (back-to-front z-order within the root tier).
    #[must_use]
    pub fn roots(&self) -> &[ViewId] {
        &self.roots
    }

    /// Return the local rectangle stored for one view.
    #[must_use]
    pub(crate) fn local_rect(&self, id: ViewId) -> Option<ViewRect> {
        self.entry(id).map(|entry| entry.local_rect)
    }

    /// Return direct child ids in sibling order (back-to-front within the tier).
    #[must_use]
    pub fn children(&self, id: ViewId) -> &[ViewId] {
        self.entry(id)
            .map(|entry| entry.children.as_slice())
            .unwrap_or(&[])
    }

    /// Return the parent of `id`, or `None` when the view is a root.
    #[must_use]
    pub fn parent(&self, id: ViewId) -> Option<ViewId> {
        self.entry(id).and_then(|entry| entry.parent)
    }

    /// Return the group's current child on the active focus path.
    #[must_use]
    pub fn current_child(&self, id: ViewId) -> Option<ViewId> {
        self.entry(id).and_then(|entry| entry.current_child)
    }

    /// Return behavior options for a retained view.
    #[must_use]
    pub fn options(&self, id: ViewId) -> Option<ViewOptions> {
        self.entry(id).map(|entry| entry.options)
    }

    /// Replace behavior options for a retained view.
    pub fn set_options(&mut self, id: ViewId, options: ViewOptions) -> bool {
        let Some(entry) = self.entry_mut(id) else {
            return false;
        };
        entry.options = options;
        self.ensure_valid_focus();
        true
    }

    /// Return resolved state for a retained view.
    #[must_use]
    pub fn state(&self, id: ViewId) -> Option<ViewState> {
        self.resolved(id).map(|view| view.state)
    }

    /// Set whether a retained view and its descendants are visible.
    pub fn set_visible(&mut self, id: ViewId, visible: bool) -> bool {
        let Some(entry) = self.entry_mut(id) else {
            return false;
        };
        entry.state.visible = visible;
        self.ensure_valid_focus();
        true
    }

    /// Set whether a retained view can receive input and focus.
    pub fn set_enabled(&mut self, id: ViewId, enabled: bool) -> bool {
        let Some(entry) = self.entry_mut(id) else {
            return false;
        };
        entry.state.enabled = enabled;
        self.ensure_valid_focus();
        true
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

    /// Return the ancestor chain for `id`, starting with `id` itself and ending at the root.
    #[must_use]
    pub fn ancestors_inclusive(&self, id: ViewId) -> Vec<ViewId> {
        let mut ids = Vec::new();
        let mut current = Some(id);

        while let Some(view_id) = current {
            let Some(entry) = self.entry(view_id) else {
                break;
            };
            ids.push(view_id);
            current = entry.parent;
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
        self.resolved_paint_order()
            .into_iter()
            .rev()
            .find_map(|view| {
                scope
                    .is_none_or(|scope_ids| scope_ids.contains(&view.id))
                    .then_some(view)
                    .filter(|view| view.clip.is_some_and(|clip| clip.contains_point(x, y)))
                    .map(|view| view.id)
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
        self.focused = None;
        self.pointer_capture = None;
    }

    fn allocate_id(&mut self) -> ViewId {
        let start = self.next_id;
        loop {
            let id = ViewId(self.next_id);
            self.next_id = self.next_id.wrapping_add(1);
            if self.entry(id).is_none() {
                return id;
            }
            assert_ne!(
                self.next_id, start,
                "View id space exhausted; close unused views before registering more"
            );
        }
    }

    pub(super) fn entry(&self, id: ViewId) -> Option<&ViewEntry> {
        self.views.iter().find(|entry| entry.id == id)
    }

    pub(super) fn entry_mut(&mut self, id: ViewId) -> Option<&mut ViewEntry> {
        self.views.iter_mut().find(|entry| entry.id == id)
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

    /// Return the local-coordinate origin for children of `parent_id`.
    ///
    /// Frame-root children use view-space coordinates (matching [`super::geometry`] resolution).
    /// All other parents use the parent's resolved top-left corner.
    fn parent_local_origin(&self, parent_id: ViewId) -> Option<(i64, i64)> {
        if let Some(frame) = self.frame_roots.get(&parent_id) {
            let view = frame.geometry.view;
            let ox = frame.scroll_x.offset() as i64;
            let oy = frame.scroll_y.offset() as i64;
            Some((view.x - ox, view.y - oy))
        } else {
            self.rect(parent_id).map(|rect| (rect.x, rect.y))
        }
    }
}

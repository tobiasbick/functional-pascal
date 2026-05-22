use super::{ViewEntry, ViewId, ViewRect, ViewRegistry};

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
}

fn rect_contains_point(rect: ViewRect, x: i64, y: i64) -> bool {
    let max_x = rect.x.saturating_add(rect.width.max(0));
    let max_y = rect.y.saturating_add(rect.height.max(0));
    x >= rect.x && y >= rect.y && x < max_x && y < max_y
}

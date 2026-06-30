//! Resolve local view geometry, ancestor transforms, and effective clips.

use super::{ResolvedView, ViewId, ViewRect, ViewRegistry};

#[derive(Clone, Copy)]
enum ClipLimit {
    Unbounded,
    Hidden,
    Rect(ViewRect),
}

impl ViewRegistry {
    /// Resolve one retained node to absolute geometry and an effective clip.
    #[must_use]
    pub fn resolved(&self, id: ViewId) -> Option<ResolvedView> {
        self.resolve_with_limit(id).map(|(view, _)| view)
    }

    /// Resolve visible/exposed views in back-to-front tree order.
    #[must_use]
    pub fn resolved_paint_order(&self) -> Vec<ResolvedView> {
        self.paint_order()
            .into_iter()
            .filter_map(|id| self.resolved(id))
            .filter(|view| view.state.exposed)
            .collect()
    }

    /// Resolve every view in `root`'s subtree in paint order.
    ///
    /// The snapshot preserves hidden and fully clipped nodes so callers that track damage across
    /// state transitions can still dirty their absolute rectangles.
    #[must_use]
    pub fn resolved_subtree(&self, root: ViewId) -> Vec<ResolvedView> {
        self.subtree_ids(root)
            .into_iter()
            .filter_map(|id| self.resolved(id))
            .collect()
    }

    /// Return the effective visible clip for one view.
    #[must_use]
    pub fn clip(&self, id: ViewId) -> Option<ViewRect> {
        self.resolved(id).and_then(|view| view.clip)
    }

    fn resolve_with_limit(&self, id: ViewId) -> Option<(ResolvedView, ClipLimit)> {
        let entry = self.entry(id)?;
        let (rect, inherited_visible, inherited_limit) = match entry.parent {
            Some(parent_id) => {
                let (parent, child_limit) = self.resolve_with_limit(parent_id)?;
                let rect = ViewRect {
                    x: parent.content_origin.0.saturating_add(entry.local_rect.x),
                    y: parent.content_origin.1.saturating_add(entry.local_rect.y),
                    width: entry.local_rect.width,
                    height: entry.local_rect.height,
                };
                (rect, parent.state.visible, child_limit)
            }
            None => (entry.local_rect, true, ClipLimit::Unbounded),
        };

        let visible = inherited_visible && entry.state.visible;
        let clip = if visible {
            match inherited_limit {
                ClipLimit::Unbounded => (!rect.is_empty()).then_some(rect),
                ClipLimit::Hidden => None,
                ClipLimit::Rect(limit) => rect.intersection(limit),
            }
        } else {
            None
        };

        let content_origin = (rect.x, rect.y);
        let child_limit = if !visible {
            ClipLimit::Hidden
        } else if entry.options.clip_children {
            clip.map_or(ClipLimit::Hidden, ClipLimit::Rect)
        } else {
            inherited_limit
        };

        let focused = self.focused == Some(id);
        let active = focused
            || self
                .focused
                .is_some_and(|focused_id| self.is_ancestor(id, focused_id));
        let mut state = entry.state;
        state.visible = visible;
        state.focused = focused;
        state.active = active;
        state.exposed = clip.is_some();

        Some((
            ResolvedView {
                id,
                rect,
                content_origin,
                clip,
                state,
                options: entry.options,
            },
            child_limit,
        ))
    }

    fn is_ancestor(&self, candidate: ViewId, descendant: ViewId) -> bool {
        let mut current = self.parent(descendant);
        while let Some(id) = current {
            if id == candidate {
                return true;
            }
            current = self.parent(id);
        }
        false
    }
}

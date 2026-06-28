//! Anchor/grow layout for retained views.
//!
//! Spec: `docs/pascal/std/tui/app/views.md`

use super::{ViewId, ViewRect, ViewRegistry};

/// Layout flags controlling how a view tracks parent size changes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ViewLayout {
    /// Pin the left edge to the parent's left edge plus [`margin_left`](Self::margin_left).
    pub anchor_left: bool,
    /// Pin the top edge to the parent's top edge plus [`margin_top`](Self::margin_top).
    pub anchor_top: bool,
    /// Pin the right edge to the parent's right edge minus [`margin_right`](Self::margin_right).
    pub anchor_right: bool,
    /// Pin the bottom edge to the parent's bottom edge minus [`margin_bottom`](Self::margin_bottom).
    pub anchor_bottom: bool,
    /// Offset from the parent's left edge when [`anchor_left`](Self::anchor_left) is set.
    pub margin_left: i64,
    /// Offset from the parent's top edge when [`anchor_top`](Self::anchor_top) is set.
    pub margin_top: i64,
    /// Offset from the parent's right edge when [`anchor_right`](Self::anchor_right) is set.
    pub margin_right: i64,
    /// Offset from the parent's bottom edge when [`anchor_bottom`](Self::anchor_bottom) is set.
    pub margin_bottom: i64,
}

impl ViewLayout {
    /// Returns `true` when any anchor edge is set.
    #[must_use]
    pub fn is_active(self) -> bool {
        self.anchor_left || self.anchor_top || self.anchor_right || self.anchor_bottom
    }
}

/// Resolve a local rectangle from parent bounds and layout flags.
#[must_use]
pub fn resolve_layout(parent: ViewRect, current: ViewRect, layout: ViewLayout) -> ViewRect {
    if !layout.is_active() {
        return current;
    }

    let inner_left = parent.x;
    let inner_top = parent.y;
    let inner_width = parent.width.max(0);
    let inner_height = parent.height.max(0);

    let width = if layout.anchor_left && layout.anchor_right {
        (inner_width - layout.margin_left - layout.margin_right).max(0)
    } else {
        current.width
    };

    let height = if layout.anchor_top && layout.anchor_bottom {
        (inner_height - layout.margin_top - layout.margin_bottom).max(0)
    } else {
        current.height
    };

    let x = if layout.anchor_left && layout.anchor_right {
        inner_left + layout.margin_left
    } else if layout.anchor_right && !layout.anchor_left {
        inner_left + inner_width - layout.margin_right - width
    } else if layout.anchor_left {
        inner_left + layout.margin_left
    } else {
        current.x
    };

    let y = if layout.anchor_top && layout.anchor_bottom {
        inner_top + layout.margin_top
    } else if layout.anchor_bottom && !layout.anchor_top {
        inner_top + inner_height - layout.margin_bottom - height
    } else if layout.anchor_top {
        inner_top + layout.margin_top
    } else {
        current.y
    };

    ViewRect {
        x,
        y,
        width,
        height,
    }
}

impl ViewRegistry {
    /// Return retained layout flags for one view.
    #[must_use]
    pub fn layout(&self, id: ViewId) -> Option<ViewLayout> {
        self.entry(id).map(|entry| entry.layout)
    }

    /// Replace layout flags for one retained view.
    pub fn set_layout(&mut self, id: ViewId, layout: ViewLayout) -> bool {
        let Some(entry) = self.entry_mut(id) else {
            return false;
        };
        entry.layout = layout;
        true
    }

    /// Recompute every anchored view from root bounds downward.
    ///
    /// `terminal_bounds` supplies the parent rectangle for root views.
    pub fn relayout_all_roots(&mut self, terminal_bounds: ViewRect) -> bool {
        let roots = self.roots.to_vec();
        let mut changed = false;
        for root in roots {
            changed |= self.relayout_subtree(root, terminal_bounds);
        }
        changed
    }

    /// Recompute anchored layout for `id` and its descendants using the current parent bounds.
    pub fn relayout_from_view(&mut self, id: ViewId, terminal_bounds: ViewRect) -> bool {
        let parent_bounds = match self.parent(id) {
            None => terminal_bounds,
            Some(parent) => self.layout_bounds_for_children(parent),
        };
        self.relayout_subtree(id, parent_bounds)
    }

    /// Recompute anchored layout for `id` and its descendants.
    pub fn relayout_subtree(&mut self, id: ViewId, parent_bounds: ViewRect) -> bool {
        let Some(entry) = self.entry(id) else {
            return false;
        };
        let layout = entry.layout;
        let current = entry.local_rect;
        let next = resolve_layout(parent_bounds, current, layout);
        let mut changed = false;
        if next != current {
            if let Some(entry) = self.entry_mut(id) {
                entry.local_rect = next;
            }
            changed = true;
        }

        let child_bounds = self.layout_bounds_for_children(id);
        for child in self.children(id).to_vec() {
            changed |= self.relayout_subtree(child, child_bounds);
        }
        changed
    }

    /// Recompute anchored layout for every direct child of `parent_id`.
    pub fn relayout_children(&mut self, parent_id: ViewId) -> bool {
        let child_bounds = self.layout_bounds_for_children(parent_id);
        let mut changed = false;
        for child in self.children(parent_id).to_vec() {
            changed |= self.relayout_subtree(child, child_bounds);
        }
        changed
    }

    /// Parent bounds used to lay out direct children of `parent_id`.
    fn layout_bounds_for_children(&self, parent_id: ViewId) -> ViewRect {
        if let Some(frame) = self.frame_roots.get(&parent_id) {
            let view = frame.geometry.view;
            ViewRect {
                x: 0,
                y: 0,
                width: view.width,
                height: view.height,
            }
        } else {
            self.local_rect(parent_id).map_or(
                ViewRect {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
                |rect| ViewRect {
                    x: 0,
                    y: 0,
                    width: rect.width,
                    height: rect.height,
                },
            )
        }
    }
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
    fn resolve_layout_stretches_horizontal_bar() {
        let parent = rect(0, 0, 100, 24);
        let current = rect(0, 0, 80, 1);
        let layout = ViewLayout {
            anchor_left: true,
            anchor_top: true,
            anchor_right: true,
            ..ViewLayout::default()
        };
        assert_eq!(resolve_layout(parent, current, layout), rect(0, 0, 100, 1));
    }

    #[test]
    fn resolve_layout_fills_center_with_margins() {
        let parent = rect(0, 0, 100, 30);
        let current = rect(0, 1, 80, 22);
        let layout = ViewLayout {
            anchor_left: true,
            anchor_top: true,
            anchor_right: true,
            anchor_bottom: true,
            margin_top: 1,
            margin_bottom: 1,
            ..ViewLayout::default()
        };
        assert_eq!(resolve_layout(parent, current, layout), rect(0, 1, 100, 28));
    }

    #[test]
    fn relayout_all_roots_updates_shell_like_views() {
        let mut registry = ViewRegistry::default();
        let menu = registry.register(rect(0, 0, 80, 1));
        registry.set_layout(
            menu,
            ViewLayout {
                anchor_left: true,
                anchor_top: true,
                anchor_right: true,
                ..ViewLayout::default()
            },
        );
        let desktop = registry.register(rect(0, 1, 80, 22));
        registry.set_layout(
            desktop,
            ViewLayout {
                anchor_left: true,
                anchor_top: true,
                anchor_right: true,
                anchor_bottom: true,
                margin_top: 1,
                margin_bottom: 1,
                ..ViewLayout::default()
            },
        );
        let status = registry.register(rect(0, 23, 80, 1));
        registry.set_layout(
            status,
            ViewLayout {
                anchor_left: true,
                anchor_right: true,
                anchor_bottom: true,
                ..ViewLayout::default()
            },
        );

        registry.relayout_all_roots(rect(0, 0, 100, 30));

        assert_eq!(registry.rect(menu), Some(rect(0, 0, 100, 1)));
        assert_eq!(registry.rect(desktop), Some(rect(0, 1, 100, 28)));
        assert_eq!(registry.rect(status), Some(rect(0, 29, 100, 1)));
    }
}

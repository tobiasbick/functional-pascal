//! Captured frame move, resize, zoom, and next-window activation.
//!
//! Plan: `docs/future/windows-dialogs/README.md`
//! Review: `docs/future/windows-dialogs/TUI-CODE-REVIEW.md`

use super::hit::{FrameChromeHit, frame_chrome_hit};
use super::state::{FrameResizeEdge, WindowInteractionKind};
use crate::{RootActivation, ViewId, ViewRect, ViewRegistry};

/// In-flight pointer interaction on a frame root.
pub use super::state::WindowInteraction;

impl ViewRegistry {
    /// Activate the next window root in z-order, skipping `exclude` roots (for example modals).
    ///
    /// Cycles from the current active root toward the back and wraps to the front. Returns `None`
    /// when no eligible roots exist.
    pub fn activate_next_root_excluding(&mut self, exclude: &[ViewId]) -> Option<RootActivation> {
        let eligible: Vec<ViewId> = self
            .roots()
            .iter()
            .copied()
            .filter(|root| !exclude.contains(root))
            .collect();
        if eligible.is_empty() {
            return None;
        }

        let active = self.active_root().filter(|root| eligible.contains(root));
        let active_index = active
            .and_then(|active| eligible.iter().position(|root| *root == active))
            .unwrap_or_else(|| eligible.len().saturating_sub(1));
        let next_index = if active_index == 0 {
            eligible.len() - 1
        } else {
            active_index - 1
        };
        let next = eligible[next_index];
        self.activate_root(next)
    }

    /// Hit-test frame chrome for move or resize on the root containing `id`.
    #[must_use]
    pub fn frame_chrome_hit_at(&self, id: ViewId, x: i64, y: i64) -> FrameChromeHit {
        let Some(state) = self.frame_root_state(id) else {
            return FrameChromeHit::None;
        };
        frame_chrome_hit(
            &state.geometry,
            state.capabilities.movable,
            state.capabilities.resizable,
            x,
            y,
        )
    }

    /// Begin a captured title-bar move when `id` is a movable frame root and `(x, y)` hits chrome.
    pub fn begin_frame_move(&mut self, id: ViewId, x: i64, y: i64) -> bool {
        let root = match self.frame_root_of(id) {
            Some(root) => root,
            None => return false,
        };
        if !self
            .frame_root_state(root)
            .is_some_and(|state| state.capabilities.movable)
            || !matches!(self.frame_chrome_hit_at(root, x, y), FrameChromeHit::Move)
        {
            return false;
        }
        let rect = match self.rect(root) {
            Some(rect) => rect,
            None => return false,
        };
        self.window_interaction = Some(WindowInteraction {
            root,
            kind: WindowInteractionKind::Move {
                grab_x: x.saturating_sub(rect.x),
                grab_y: y.saturating_sub(rect.y),
            },
        });
        self.capture_pointer(root)
    }

    /// Begin a captured border resize when `id` is a resizable frame root and `(x, y)` hits chrome.
    pub fn begin_frame_resize(&mut self, id: ViewId, x: i64, y: i64) -> bool {
        let root = match self.frame_root_of(id) {
            Some(root) => root,
            None => return false,
        };
        let FrameChromeHit::Resize(edge) = self.frame_chrome_hit_at(root, x, y) else {
            return false;
        };
        if !self
            .frame_root_state(root)
            .is_some_and(|state| state.capabilities.resizable)
        {
            return false;
        }
        let anchor = match self.rect(root) {
            Some(rect) => rect,
            None => return false,
        };
        self.window_interaction = Some(WindowInteraction {
            root,
            kind: WindowInteractionKind::Resize {
                edge,
                anchor,
                start_x: x,
                start_y: y,
            },
        });
        self.capture_pointer(root)
    }

    /// Apply a captured move or resize drag at `(x, y)`.
    pub fn drag_frame_interaction(&mut self, x: i64, y: i64) -> bool {
        let Some(interaction) = self.window_interaction else {
            return false;
        };
        match interaction.kind {
            WindowInteractionKind::Move { grab_x, grab_y } => {
                let Some(rect) = self.rect(interaction.root) else {
                    return false;
                };
                let candidate = ViewRect {
                    x: x.saturating_sub(grab_x),
                    y: y.saturating_sub(grab_y),
                    width: rect.width,
                    height: rect.height,
                };
                self.set_root_rect_constrained(interaction.root, candidate);
                self.refresh_frame_geometry(interaction.root);
                true
            }
            WindowInteractionKind::Resize {
                edge,
                anchor,
                start_x,
                start_y,
            } => {
                let delta_x = x.saturating_sub(start_x);
                let delta_y = y.saturating_sub(start_y);
                let candidate = resize_rect(anchor, edge, delta_x, delta_y);
                self.set_root_rect_constrained(interaction.root, candidate);
                self.refresh_frame_geometry(interaction.root);
                true
            }
        }
    }

    /// End a captured frame interaction and release pointer capture.
    pub fn end_frame_interaction(&mut self) -> bool {
        let had = self.window_interaction.is_some();
        self.window_interaction = None;
        if self.captured_pointer().is_some() {
            self.release_pointer();
        }
        had
    }

    /// Zoom a zoomable frame root to the desktop work area.
    ///
    /// Returns `false` when the root is unknown, not zoomable, already zoomed, or no work area is
    /// configured.
    pub fn zoom_frame_root(&mut self, id: ViewId) -> bool {
        let Some(root) = self.frame_root_of(id) else {
            return false;
        };
        let Some(work_area) = self.desktop_metrics().work_area else {
            return false;
        };
        let Some(current) = self.rect(root) else {
            return false;
        };
        let Some(state) = self.frame_roots.get_mut(&root) else {
            return false;
        };
        if !state.capabilities.zoomable || state.pre_zoom_rect.is_some() {
            return false;
        }
        state.pre_zoom_rect = Some(current);
        let zoomed = self.constrain_window_rect(work_area);
        self.set_rect(root, zoomed);
        self.refresh_frame_geometry(root);
        true
    }

    /// Restore a zoomed frame root to its pre-zoom rectangle.
    pub fn restore_frame_root(&mut self, id: ViewId) -> bool {
        let Some(root) = self.frame_root_of(id) else {
            return false;
        };
        let Some(saved) = self
            .frame_roots
            .get_mut(&root)
            .and_then(|state| state.pre_zoom_rect.take())
        else {
            return false;
        };
        if self.set_root_rect_constrained(root, saved).is_none() {
            return false;
        }
        self.refresh_frame_geometry(root);
        true
    }

    /// Return whether the frame root containing `id` is currently zoomed.
    #[must_use]
    pub fn frame_is_zoomed(&self, id: ViewId) -> bool {
        self.frame_root_state(id)
            .is_some_and(|state| state.pre_zoom_rect.is_some())
    }
}

fn resize_rect(anchor: ViewRect, edge: FrameResizeEdge, delta_x: i64, delta_y: i64) -> ViewRect {
    match edge {
        FrameResizeEdge::East => ViewRect {
            width: anchor.width.saturating_add(delta_x).max(1),
            ..anchor
        },
        FrameResizeEdge::West => {
            let width = anchor.width.saturating_sub(delta_x).max(1);
            ViewRect {
                x: anchor.x.saturating_add(anchor.width.saturating_sub(width)),
                width,
                ..anchor
            }
        }
        FrameResizeEdge::South => ViewRect {
            height: anchor.height.saturating_add(delta_y).max(1),
            ..anchor
        },
        FrameResizeEdge::SouthEast => ViewRect {
            width: anchor.width.saturating_add(delta_x).max(1),
            height: anchor.height.saturating_add(delta_y).max(1),
            ..anchor
        },
        FrameResizeEdge::SouthWest => {
            let width = anchor.width.saturating_sub(delta_x).max(1);
            ViewRect {
                x: anchor.x.saturating_add(anchor.width.saturating_sub(width)),
                width,
                height: anchor.height.saturating_add(delta_y).max(1),
                y: anchor.y,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FrameCapabilities, FrameContentSize, FrameKind, FrameRootSpec};

    fn rect(x: i64, y: i64, width: i64, height: i64) -> ViewRect {
        ViewRect {
            x,
            y,
            width,
            height,
        }
    }

    fn movable_window_spec(outer: ViewRect) -> FrameRootSpec {
        FrameRootSpec {
            kind: FrameKind::Window,
            outer,
            content_size: FrameContentSize::new(0, 0),
            capabilities: FrameCapabilities {
                movable: true,
                resizable: true,
                zoomable: true,
                closable: false,
                scrollable: false,
            },
            options: Default::default(),
        }
    }

    #[test]
    fn activate_next_root_cycles_toward_back_and_wraps() {
        let mut registry = ViewRegistry::default();
        assert!(registry.set_desktop_work_area(rect(0, 0, 80, 25)));
        let first = registry
            .register_frame_root(movable_window_spec(rect(2, 2, 20, 8)))
            .expect("first frame");
        let second = registry
            .register_frame_root(movable_window_spec(rect(30, 4, 20, 8)))
            .expect("second frame");
        let first_leaf = registry.register(rect(4, 4, 4, 1));
        assert!(registry.set_parent(first_leaf, Some(first.view_id)));
        let second_leaf = registry.register(rect(32, 6, 4, 1));
        assert!(registry.set_parent(second_leaf, Some(second.view_id)));
        assert!(registry.set_options(
            second_leaf,
            crate::ViewOptions {
                selectable: true,
                tab_stop: true,
                ..Default::default()
            },
        ));
        assert!(registry.set_options(
            first_leaf,
            crate::ViewOptions {
                selectable: true,
                tab_stop: true,
                ..Default::default()
            },
        ));

        registry.focus_view(second_leaf);
        assert_eq!(registry.active_root(), Some(second.view_id));

        let next = registry
            .activate_next_root_excluding(&[])
            .expect("next root");
        assert_eq!(next.root, first.view_id);
        assert_eq!(registry.active_root(), Some(first.view_id));

        let wrapped = registry.activate_next_root_excluding(&[]).expect("wrap");
        assert_eq!(wrapped.root, second.view_id);
    }

    #[test]
    fn captured_move_updates_root_rect_and_geometry() {
        let mut registry = ViewRegistry::default();
        assert!(registry.set_desktop_work_area(rect(0, 0, 80, 25)));
        let frame = registry
            .register_frame_root(movable_window_spec(rect(10, 4, 20, 8)))
            .expect("frame");
        let title_x = frame.geometry.title_bar.x + 5;
        let title_y = frame.geometry.title_bar.y;

        assert!(registry.begin_frame_move(frame.view_id, title_x, title_y));
        assert!(registry.drag_frame_interaction(title_x + 4, title_y + 3));
        assert!(registry.end_frame_interaction());

        assert_eq!(registry.rect(frame.view_id), Some(rect(14, 7, 20, 8)));
        assert_eq!(
            registry
                .frame_root_state(frame.view_id)
                .unwrap()
                .geometry
                .outer,
            rect(14, 7, 20, 8)
        );
    }

    #[test]
    fn zoom_and_restore_toggle_root_rect() {
        let mut registry = ViewRegistry::default();
        assert!(registry.set_desktop_work_area(rect(0, 0, 80, 25)));
        let frame = registry
            .register_frame_root(movable_window_spec(rect(10, 4, 20, 8)))
            .expect("frame");
        let original = registry.rect(frame.view_id).unwrap();

        assert!(registry.zoom_frame_root(frame.view_id));
        assert!(registry.frame_is_zoomed(frame.view_id));
        assert_eq!(registry.rect(frame.view_id), Some(rect(0, 0, 80, 25)));

        assert!(registry.restore_frame_root(frame.view_id));
        assert!(!registry.frame_is_zoomed(frame.view_id));
        assert_eq!(registry.rect(frame.view_id), Some(original));
    }
}

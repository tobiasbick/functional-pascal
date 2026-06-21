//! Optional cascade and tile layout for window-kind frame roots.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use crate::{ViewId, ViewRect, ViewRegistry};

use super::{FrameKind, FrameRootState};

impl ViewRegistry {
    /// Cascade eligible window roots diagonally inside the desktop work area.
    ///
    /// Each root keeps its current size. Returns the number of roots repositioned, or `0` when no
    /// work area is configured or no eligible roots exist. Skips zoomed roots and ids in
    /// `exclude` (for example an active modal root).
    pub fn cascade_frame_roots_excluding(
        &mut self,
        exclude: &[ViewId],
        step_x: i64,
        step_y: i64,
    ) -> usize {
        let Some(work_area) = self.desktop_metrics().work_area else {
            return 0;
        };
        let roots = self.layout_eligible_window_roots(exclude);
        if roots.is_empty() {
            return 0;
        }
        let _ = self.end_frame_interaction();
        let step_x = step_x.max(1);
        let step_y = step_y.max(0);
        for (index, root) in roots.iter().enumerate() {
            let Some(current) = self.rect(*root) else {
                continue;
            };
            let offset = index as i64;
            let candidate = ViewRect {
                x: work_area.x.saturating_add(step_x.saturating_mul(offset)),
                y: work_area.y.saturating_add(step_y.saturating_mul(offset)),
                width: current.width,
                height: current.height,
            };
            let _ = self.set_root_rect_constrained(*root, candidate);
        }
        roots.len()
    }

    /// Tile eligible window roots in a grid that fills the desktop work area.
    ///
    /// Each root is resized to its grid cell. Returns the number of roots repositioned, or `0`
    /// when no work area is configured or no eligible roots exist.
    pub fn tile_frame_roots_excluding(&mut self, exclude: &[ViewId]) -> usize {
        let Some(work_area) = self.desktop_metrics().work_area else {
            return 0;
        };
        let roots = self.layout_eligible_window_roots(exclude);
        let count = roots.len();
        if count == 0 {
            return 0;
        }
        let _ = self.end_frame_interaction();
        let count_i = count as i64;
        let cols = grid_columns(count_i);
        let rows = (count_i + cols - 1) / cols;
        let base_width = work_area.width / cols;
        let base_height = work_area.height / rows;
        let extra_width = work_area.width - base_width * cols;
        let extra_height = work_area.height - base_height * rows;
        for (index, root) in roots.iter().enumerate() {
            let index = index as i64;
            let col = index % cols;
            let row = index / cols;
            let width = if col == cols - 1 {
                base_width + extra_width
            } else {
                base_width
            };
            let height = if row == rows - 1 {
                base_height + extra_height
            } else {
                base_height
            };
            let candidate = ViewRect {
                x: work_area.x.saturating_add(base_width.saturating_mul(col)),
                y: work_area.y.saturating_add(base_height.saturating_mul(row)),
                width: width.max(1),
                height: height.max(1),
            };
            let _ = self.set_root_rect_constrained(*root, candidate);
        }
        count
    }

    fn layout_eligible_window_roots(&self, exclude: &[ViewId]) -> Vec<ViewId> {
        self.roots()
            .iter()
            .copied()
            .filter(|root| !exclude.contains(root))
            .filter(|root| self.is_layout_eligible_window_root(*root))
            .collect()
    }

    fn is_layout_eligible_window_root(&self, root: ViewId) -> bool {
        self.frame_roots
            .get(&root)
            .is_some_and(|state| state.is_layout_eligible())
    }
}

impl FrameRootState {
    fn is_layout_eligible(&self) -> bool {
        self.kind == FrameKind::Window && self.pre_zoom_rect.is_none()
    }
}

fn grid_columns(count: i64) -> i64 {
    if count <= 1 {
        return 1;
    }
    let mut cols = 1i64;
    while cols.saturating_mul(cols) < count {
        cols += 1;
    }
    cols
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FrameCapabilities, FrameContentSize, FrameRootSpec};

    fn rect(x: i64, y: i64, width: i64, height: i64) -> ViewRect {
        ViewRect {
            x,
            y,
            width,
            height,
        }
    }

    fn window_spec(outer: ViewRect) -> FrameRootSpec {
        FrameRootSpec {
            kind: FrameKind::Window,
            outer,
            content_size: FrameContentSize::new(0, 0),
            capabilities: FrameCapabilities {
                movable: true,
                resizable: true,
                zoomable: false,
                closable: false,
                scrollable: false,
            },
            options: Default::default(),
        }
    }

    fn register_window(registry: &mut ViewRegistry, outer: ViewRect) -> ViewId {
        registry
            .register_frame_root(window_spec(outer))
            .expect("window")
            .view_id
    }

    #[test]
    fn cascade_offsets_window_roots_from_work_area_origin() {
        let mut registry = ViewRegistry::default();
        assert!(registry.set_desktop_work_area(rect(0, 0, 80, 25)));
        let first = register_window(&mut registry, rect(30, 10, 20, 8));
        let second = register_window(&mut registry, rect(40, 12, 18, 6));

        let laid_out = registry.cascade_frame_roots_excluding(&[], 2, 1);
        assert_eq!(laid_out, 2);
        assert_eq!(registry.rect(first), Some(rect(0, 0, 20, 8)));
        assert_eq!(registry.rect(second), Some(rect(2, 1, 18, 6)));
    }

    #[test]
    fn tile_resizes_window_roots_into_work_area_grid() {
        let mut registry = ViewRegistry::default();
        assert!(registry.set_desktop_work_area(rect(0, 0, 80, 24)));
        let first = register_window(&mut registry, rect(1, 1, 20, 8));
        let second = register_window(&mut registry, rect(5, 2, 20, 8));
        let third = register_window(&mut registry, rect(9, 3, 20, 8));

        let laid_out = registry.tile_frame_roots_excluding(&[]);
        assert_eq!(laid_out, 3);
        assert_eq!(registry.rect(first), Some(rect(0, 0, 40, 12)));
        assert_eq!(registry.rect(second), Some(rect(40, 0, 40, 12)));
        assert_eq!(registry.rect(third), Some(rect(0, 12, 40, 12)));
    }

    #[test]
    fn layout_skips_zoomed_and_dialog_roots() {
        let mut registry = ViewRegistry::default();
        assert!(registry.set_desktop_work_area(rect(0, 0, 80, 25)));
        let window = register_window(&mut registry, rect(10, 4, 20, 8));
        let dialog = registry
            .register_frame_root(FrameRootSpec {
                kind: FrameKind::Dialog,
                outer: rect(12, 6, 20, 8),
                content_size: FrameContentSize::new(0, 0),
                capabilities: FrameCapabilities::plain(),
                options: Default::default(),
            })
            .expect("dialog")
            .view_id;
        registry.frame_roots.get_mut(&window).unwrap().capabilities.zoomable = true;
        assert!(registry.zoom_frame_root(window));

        assert_eq!(registry.cascade_frame_roots_excluding(&[], 2, 1), 0);
        assert_eq!(registry.tile_frame_roots_excluding(&[]), 0);
        assert_eq!(registry.rect(dialog), Some(rect(12, 6, 20, 8)));
    }
}

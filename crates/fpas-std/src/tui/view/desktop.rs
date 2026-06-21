//! Desktop/window-manager geometry for retained TUI roots.
//!
//! The frame widget will consume these primitives for desktop-bounded placement, active/inactive
//! palette selection, and shadow damage/hit-test regions.
//!
//! Spec: `docs/pascal/std/tui/app/README.md`

use super::{ViewId, ViewRect, ViewRegistry};

/// Active/inactive palette state for a window root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowPalette {
    /// The root contains the focused view and should use the active frame colors.
    Active,
    /// The root is not on the active focus path and should use inactive frame colors.
    Inactive,
}

/// Desktop placement constraints shared by frame roots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopMetrics {
    /// Optional desktop work area; frame roots are constrained to this rectangle when present.
    pub work_area: Option<ViewRect>,
    /// Minimum frame root width in terminal cells.
    pub min_window_width: i64,
    /// Minimum frame root height in terminal cells.
    pub min_window_height: i64,
    /// Whether frame roots should produce shadow regions.
    pub shadow_enabled: bool,
}

impl Default for DesktopMetrics {
    fn default() -> Self {
        Self {
            work_area: None,
            min_window_width: 1,
            min_window_height: 1,
            shadow_enabled: false,
        }
    }
}

/// L-shaped shadow geometry for a window root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowShadow {
    /// Right-side shadow column, clipped to the desktop work area when one is configured.
    pub right: Option<ViewRect>,
    /// Bottom shadow row, clipped to the desktop work area when one is configured.
    pub bottom: Option<ViewRect>,
}

impl WindowShadow {
    /// Return `true` when both shadow regions are clipped away.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.right.is_none() && self.bottom.is_none()
    }
}

impl ViewRegistry {
    /// Return current desktop placement metrics.
    #[must_use]
    pub fn desktop_metrics(&self) -> DesktopMetrics {
        self.desktop
    }

    /// Set the desktop work area used to constrain root window placement.
    ///
    /// Returns `false` and leaves the previous work area unchanged when `work_area` is empty.
    pub fn set_desktop_work_area(&mut self, work_area: ViewRect) -> bool {
        if work_area.is_empty() {
            return false;
        }
        self.desktop.work_area = Some(work_area);
        true
    }

    /// Clear any configured desktop work area.
    pub fn clear_desktop_work_area(&mut self) {
        self.desktop.work_area = None;
    }

    /// Set the minimum window root size.
    ///
    /// Values below one cell are clamped to one.
    pub fn set_min_window_size(&mut self, width: i64, height: i64) {
        self.desktop.min_window_width = width.max(1);
        self.desktop.min_window_height = height.max(1);
    }

    /// Enable or disable frame-root shadow geometry.
    pub fn set_window_shadow_enabled(&mut self, enabled: bool) {
        self.desktop.shadow_enabled = enabled;
    }

    /// Return the active/inactive palette state for the root containing `id`.
    ///
    /// Returns `None` for unknown view ids.
    #[must_use]
    pub fn root_palette(&self, id: ViewId) -> Option<WindowPalette> {
        let root = self.root_of(id)?;
        Some(if self.active_root() == Some(root) {
            WindowPalette::Active
        } else {
            WindowPalette::Inactive
        })
    }

    /// Constrain a candidate frame-root rectangle to the desktop metrics.
    #[must_use]
    pub fn constrain_window_rect(&self, rect: ViewRect) -> ViewRect {
        let mut width = rect.width.max(self.desktop.min_window_width);
        let mut height = rect.height.max(self.desktop.min_window_height);
        let Some(work_area) = self.desktop.work_area else {
            return ViewRect {
                x: rect.x,
                y: rect.y,
                width,
                height,
            };
        };

        width = width.min(work_area.width).max(1);
        height = height.min(work_area.height).max(1);

        ViewRect {
            x: clamp_origin(rect.x, work_area.x, work_area.width, width),
            y: clamp_origin(rect.y, work_area.y, work_area.height, height),
            width,
            height,
        }
    }

    /// Apply desktop constraints to the root containing `id` and return the stored root rectangle.
    ///
    /// Returns `None` for unknown view ids.
    pub fn set_root_rect_constrained(&mut self, id: ViewId, rect: ViewRect) -> Option<ViewRect> {
        let root = self.root_of(id)?;
        let constrained = self.constrain_window_rect(rect);
        self.set_rect(root, constrained);
        self.refresh_frame_geometry(root);
        Some(constrained)
    }

    /// Return frame-root shadow geometry for the root containing `id`.
    ///
    /// Returns `None` when the view is unknown or shadows are disabled.
    #[must_use]
    pub fn root_shadow(&self, id: ViewId) -> Option<WindowShadow> {
        if !self.desktop.shadow_enabled {
            return None;
        }
        let root = self.root_of(id)?;
        self.rect(root).map(|rect| self.window_shadow(rect))
    }

    /// Compute frame shadow geometry for a rectangle without looking up a view.
    #[must_use]
    pub fn window_shadow(&self, rect: ViewRect) -> WindowShadow {
        if rect.is_empty() {
            return WindowShadow {
                right: None,
                bottom: None,
            };
        }

        let right = ViewRect {
            x: rect.x.saturating_add(rect.width),
            y: rect.y.saturating_add(1),
            width: 1,
            height: rect.height,
        };
        let bottom = ViewRect {
            x: rect.x.saturating_add(1),
            y: rect.y.saturating_add(rect.height),
            width: rect.width,
            height: 1,
        };

        WindowShadow {
            right: self.clip_to_work_area(right),
            bottom: self.clip_to_work_area(bottom),
        }
    }

    fn clip_to_work_area(&self, rect: ViewRect) -> Option<ViewRect> {
        match self.desktop.work_area {
            Some(work_area) => rect.intersection(work_area),
            None => (!rect.is_empty()).then_some(rect),
        }
    }
}

fn clamp_origin(origin: i64, area_origin: i64, area_size: i64, size: i64) -> i64 {
    let max_origin = area_origin.saturating_add(area_size.saturating_sub(size));
    origin.clamp(area_origin, max_origin)
}

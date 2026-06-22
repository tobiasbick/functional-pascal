//! Retained painted frame widget.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use crate::{Console, DamageRegion, ViewRect};

use super::{
    FrameCapabilities, FrameContentSize, FrameGeometry, FrameKind, FrameStyle, chrome, scroll,
};

/// Window or dialog widget that paints client underlay and frame chrome overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameWidget {
    /// Text painted into the title-bar slot.
    pub title: String,
    /// Window/dialog palette preset.
    pub kind: FrameKind,
    /// Geometry and interaction capabilities.
    pub capabilities: FrameCapabilities,
    /// Logical content size used for frame geometry.
    pub content_size: FrameContentSize,
    /// Current colors.
    pub style: FrameStyle,
    /// Whether the frame belongs to the active focus root.
    pub active: bool,
    /// Horizontal scroll model mirrored from frame-root state during paint.
    pub scroll_x: crate::ScrollModel,
    /// Vertical scroll model mirrored from frame-root state during paint.
    pub scroll_y: crate::ScrollModel,
}

impl FrameWidget {
    /// Create a frame widget with the built-in palette for `kind`.
    #[must_use]
    pub fn new(
        title: String,
        kind: FrameKind,
        capabilities: FrameCapabilities,
        content_size: FrameContentSize,
    ) -> Self {
        Self {
            title,
            kind,
            capabilities,
            content_size,
            style: FrameStyle::for_kind(kind),
            active: false,
            scroll_x: crate::ScrollModel::default(),
            scroll_y: crate::ScrollModel::default(),
        }
    }

    /// Paint the client area before local handlers and descendants.
    pub fn paint_underlay(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        if let Ok(geometry) = FrameGeometry::resolve(rect, self.content_size, self.capabilities) {
            chrome::paint_underlay(console, geometry, damage, self.style);
        }
    }

    /// Paint border, title, buttons, and scroll bars after descendants.
    pub fn paint_overlay(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        if let Ok(geometry) = FrameGeometry::resolve(rect, self.content_size, self.capabilities) {
            chrome::paint_overlay(
                console,
                geometry,
                damage,
                &self.title,
                self.style,
                self.active,
            );
            if self.capabilities.scrollable {
                scroll::paint_scrollbars(
                    console,
                    geometry,
                    self.scroll_x,
                    self.scroll_y,
                    self.style,
                    damage,
                );
            }
        }
    }
}

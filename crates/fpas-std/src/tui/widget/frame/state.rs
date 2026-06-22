//! Frame-root metadata and in-flight window interaction state.
//!
//! Plan: `docs/future/windows-dialogs/README.md`

use crate::{ScrollModel, ViewId, ViewRect, ViewRegistry};

use super::scroll::sync_frame_scroll_extents;
use super::{FrameCapabilities, FrameContentSize, FrameGeometry, FrameGeometryError, FrameKind};

/// Captured frame scroll-bar thumb drag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameScrollInteraction {
    /// Frame root owning the scroll bar.
    pub root: ViewId,
    /// Dragged scroll-bar axis.
    pub orientation: crate::ScrollBarOrientation,
    /// Grab offset inside the thumb track.
    pub grab: usize,
}

/// Resize edge selected by a border hit-test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameResizeEdge {
    /// Left border.
    West,
    /// Right border.
    East,
    /// Bottom border.
    South,
    /// Bottom-right corner.
    SouthEast,
    /// Bottom-left corner.
    SouthWest,
}

/// Retained state for one registered frame root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRootState {
    /// Window or dialog semantic kind.
    pub kind: FrameKind,
    /// Implemented interaction and scroll capabilities.
    pub capabilities: FrameCapabilities,
    /// Logical content size used to resolve scroll-bar visibility.
    pub content_size: FrameContentSize,
    /// Latest resolved static geometry for the root rectangle.
    pub geometry: FrameGeometry,
    /// Rectangle saved before zoom; `None` when the frame is not zoomed.
    pub pre_zoom_rect: Option<ViewRect>,
    /// Horizontal scroll offset model.
    pub scroll_x: ScrollModel,
    /// Vertical scroll offset model.
    pub scroll_y: ScrollModel,
}

/// In-flight pointer interaction on a frame root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowInteraction {
    /// Frame root being moved or resized.
    pub root: ViewId,
    /// Interaction kind and grab geometry.
    pub kind: WindowInteractionKind,
}

/// Frame pointer interaction variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowInteractionKind {
    /// Title-bar move with a fixed grab offset from the root origin.
    Move {
        /// Horizontal distance from the pointer to the root origin.
        grab_x: i64,
        /// Vertical distance from the pointer to the root origin.
        grab_y: i64,
    },
    /// Border resize anchored at the opposite edge or corner.
    Resize {
        /// Selected border or corner.
        edge: FrameResizeEdge,
        /// Root rectangle captured at pointer down.
        anchor: ViewRect,
        /// Pointer position captured at pointer down.
        start_x: i64,
        /// Pointer position captured at pointer down.
        start_y: i64,
    },
}

impl ViewRegistry {
    /// Return frame metadata for the root containing `id`, if any.
    #[must_use]
    pub fn frame_root_state(&self, id: ViewId) -> Option<&FrameRootState> {
        let root = self.root_of(id)?;
        self.frame_roots.get(&root)
    }

    /// Return the root view id when `id` belongs to a registered frame root subtree.
    #[must_use]
    pub fn frame_root_of(&self, id: ViewId) -> Option<ViewId> {
        let root = self.root_of(id)?;
        self.frame_roots.contains_key(&root).then_some(root)
    }

    /// Store frame metadata for a newly registered frame root.
    pub(crate) fn store_frame_root(
        &mut self,
        view_id: ViewId,
        kind: FrameKind,
        capabilities: FrameCapabilities,
        content_size: FrameContentSize,
        geometry: FrameGeometry,
    ) {
        self.frame_roots.insert(
            view_id,
            FrameRootState {
                kind,
                capabilities,
                content_size,
                geometry,
                pre_zoom_rect: None,
                scroll_x: ScrollModel::default(),
                scroll_y: ScrollModel::default(),
            },
        );
        if let Some(state) = self.frame_roots.get_mut(&view_id) {
            sync_frame_scroll_extents(state);
        }
    }

    /// Remove frame metadata and any in-flight interaction for views in `subtree`.
    pub(crate) fn clear_frame_roots_in_subtree(&mut self, subtree: &[ViewId]) {
        for view_id in subtree {
            self.frame_roots.remove(view_id);
        }
        if self
            .window_interaction
            .is_some_and(|interaction| subtree.contains(&interaction.root))
        {
            self.window_interaction = None;
        }
        if self
            .frame_scroll_interaction
            .is_some_and(|interaction| subtree.contains(&interaction.root))
        {
            self.frame_scroll_interaction = None;
        }
    }

    /// Recompute frame geometry after the root rectangle changes.
    ///
    /// Returns `false` when `id` is not a frame root or the new rectangle is invalid.
    pub fn refresh_frame_geometry(&mut self, id: ViewId) -> bool {
        let root = match self.frame_root_of(id) {
            Some(root) => root,
            None => return false,
        };
        let outer = match self.rect(root) {
            Some(rect) => rect,
            None => return false,
        };
        self.maybe_measure_frame_content_size(root);
        let (content_size, capabilities) = {
            let Some(state) = self.frame_roots.get(&root) else {
                return false;
            };
            (state.content_size, state.capabilities)
        };
        let geometry = match FrameGeometry::resolve(outer, content_size, capabilities) {
            Ok(geometry) => geometry,
            Err(FrameGeometryError { .. }) => return false,
        };
        if let Some(state) = self.frame_roots.get_mut(&root) {
            state.geometry = geometry;
            sync_frame_scroll_extents(state);
        }
        true
    }

    /// Return the active frame interaction, if any.
    #[must_use]
    pub fn window_interaction(&self) -> Option<WindowInteraction> {
        self.window_interaction
    }
}

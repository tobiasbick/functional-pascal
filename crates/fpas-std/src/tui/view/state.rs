//! View state, behavior options, and resolved scene-node data.

use super::{ViewId, ViewRect};

/// Mutable state associated with one retained view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewState {
    /// Whether the view participates in layout, paint, hit-testing, and focus.
    pub visible: bool,
    /// Whether the view accepts input and can hold focus.
    pub enabled: bool,
    /// Whether this view is the focused leaf.
    pub focused: bool,
    /// Whether this view belongs to the active focus path.
    pub active: bool,
    /// Whether the resolved view has at least one visible cell.
    pub exposed: bool,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            visible: true,
            enabled: true,
            focused: false,
            active: false,
            exposed: false,
        }
    }
}

/// Behavioral options controlling focus, routing, and descendant clipping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewOptions {
    /// Whether pointer interaction may move focus to this view.
    pub selectable: bool,
    /// Whether Tab and Shift+Tab include this view.
    pub tab_stop: bool,
    /// Whether the view receives routed events during capture/pre-processing.
    pub pre_process: bool,
    /// Whether the view receives routed events during bubble/post-processing.
    pub post_process: bool,
    /// Whether descendants are clipped to this view's effective clip.
    pub clip_children: bool,
}

impl Default for ViewOptions {
    fn default() -> Self {
        Self {
            selectable: false,
            tab_stop: false,
            pre_process: false,
            post_process: false,
            clip_children: true,
        }
    }
}

/// Fully resolved scene data consumed by paint, hit-test, damage, and queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedView {
    /// Opaque view handle.
    pub id: ViewId,
    /// Absolute screen rectangle before clipping.
    pub rect: ViewRect,
    /// Effective visible screen clip, or `None` when fully clipped/hidden.
    pub clip: Option<ViewRect>,
    /// Resolved view state including focus-path and exposure flags.
    pub state: ViewState,
    /// Behavior options copied from the retained node.
    pub options: ViewOptions,
}

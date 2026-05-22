//! `Std.Graph` event model and canonical enum variants.
//!
//! **Documentation:** `docs/future/std.graph/02-pascal-surface.md` (from the repository root).

use crate::ConsoleKeyEvent;

/// Canonical `Std.Graph.EventKind` variant names for semantic registration and short aliases.
///
/// **Documentation:** `docs/future/std.graph/02-pascal-surface.md` (from the repository root).
pub const GRAPH_EVENT_KIND_VARIANTS: &[&str] =
    &["CloseRequested", "Resize", "Key", "Mouse", "Wheel"];

/// Host-normalized event kind for `Std.Graph.Event`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphEventKind {
    CloseRequested,
    Resize,
    Key,
    Mouse,
    Wheel,
}

/// Host-normalized event payload for the future `Std.Graph.Event` VM bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphEvent {
    CloseRequested,
    Resize {
        width: i64,
        height: i64,
    },
    Key(ConsoleKeyEvent),
    Mouse {
        action: usize,
        button: usize,
        x: i64,
        y: i64,
        shift: bool,
        ctrl: bool,
        alt: bool,
        meta: bool,
    },
    Wheel {
        delta_x: i64,
        delta_y: i64,
        x: i64,
        y: i64,
        shift: bool,
        ctrl: bool,
        alt: bool,
        meta: bool,
    },
}

impl GraphEvent {
    /// Returns the semantic event kind for this payload.
    pub fn kind(&self) -> GraphEventKind {
        match self {
            Self::CloseRequested => GraphEventKind::CloseRequested,
            Self::Resize { .. } => GraphEventKind::Resize,
            Self::Key(_) => GraphEventKind::Key,
            Self::Mouse { .. } => GraphEventKind::Mouse,
            Self::Wheel { .. } => GraphEventKind::Wheel,
        }
    }
}

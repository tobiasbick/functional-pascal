//! `Std.Graph` event model and canonical enum variants.
//!
//! **Documentation:** `docs/future/std.graph/02-pascal-surface.md` (from the repository root).

use crate::{ConsoleKeyEvent, UiEvent, UiModifiers, UiMouse, UiResize, UiWheel};

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

    /// Projects one internal shared UI event into the public `Std.Graph` event model.
    #[must_use]
    pub(crate) fn from_ui_event(value: UiEvent) -> Option<Self> {
        match value {
            UiEvent::CloseRequested => Some(Self::CloseRequested),
            UiEvent::Resize(resize) => Some(Self::Resize {
                width: resize.width,
                height: resize.height,
            }),
            UiEvent::Key(key) => Some(Self::Key(key)),
            UiEvent::Mouse(mouse) => Some(Self::Mouse {
                action: mouse.action,
                button: mouse.button,
                x: mouse.x,
                y: mouse.y,
                shift: mouse.modifiers.shift,
                ctrl: mouse.modifiers.ctrl,
                alt: mouse.modifiers.alt,
                meta: mouse.modifiers.meta,
            }),
            UiEvent::Wheel(wheel) => Some(Self::Wheel {
                delta_x: wheel.delta_x,
                delta_y: wheel.delta_y,
                x: wheel.x,
                y: wheel.y,
                shift: wheel.modifiers.shift,
                ctrl: wheel.modifiers.ctrl,
                alt: wheel.modifiers.alt,
                meta: wheel.modifiers.meta,
            }),
            UiEvent::Paste(_) | UiEvent::FocusGained | UiEvent::FocusLost => None,
        }
    }

    /// Projects one public `Std.Graph` event into the shared internal UI event model.
    #[must_use]
    pub(crate) fn into_ui_event(self) -> UiEvent {
        match self {
            Self::CloseRequested => UiEvent::CloseRequested,
            Self::Resize { width, height } => {
                UiEvent::Resize(UiResize::new(None, None, width, height))
            }
            Self::Key(key) => UiEvent::Key(key),
            Self::Mouse {
                action,
                button,
                x,
                y,
                shift,
                ctrl,
                alt,
                meta,
            } => UiEvent::Mouse(UiMouse::new(
                action,
                button,
                x,
                y,
                UiModifiers::new(shift, ctrl, alt, meta),
            )),
            Self::Wheel {
                delta_x,
                delta_y,
                x,
                y,
                shift,
                ctrl,
                alt,
                meta,
            } => UiEvent::Wheel(UiWheel::new(
                delta_x,
                delta_y,
                x,
                y,
                UiModifiers::new(shift, ctrl, alt, meta),
            )),
        }
    }
}

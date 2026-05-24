//! Internal event model shared by terminal and graphics runtimes.
//!
//! This type reduces repeated event reshaping between runtime layers while the
//! public Pascal-facing unit APIs remain unchanged.

use crate::{ConsoleEvent, ConsoleKeyEvent, GraphEvent, TuiEvent};

/// Shared keyboard/mouse modifier flags.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl UiModifiers {
    /// Creates one modifier payload.
    #[must_use]
    pub const fn new(shift: bool, ctrl: bool, alt: bool, meta: bool) -> Self {
        Self {
            shift,
            ctrl,
            alt,
            meta,
        }
    }
}

/// Shared resize payload used by internal UI events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiResize {
    pub old_width: Option<i64>,
    pub old_height: Option<i64>,
    pub width: i64,
    pub height: i64,
}

impl UiResize {
    /// Creates one resize payload.
    #[must_use]
    pub const fn new(
        old_width: Option<i64>,
        old_height: Option<i64>,
        width: i64,
        height: i64,
    ) -> Self {
        Self {
            old_width,
            old_height,
            width,
            height,
        }
    }
}

/// Shared wheel payload used by internal UI events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiWheel {
    pub delta_x: i64,
    pub delta_y: i64,
    pub x: i64,
    pub y: i64,
    pub modifiers: UiModifiers,
}

impl UiWheel {
    /// Creates one wheel payload.
    #[must_use]
    pub const fn new(delta_x: i64, delta_y: i64, x: i64, y: i64, modifiers: UiModifiers) -> Self {
        Self {
            delta_x,
            delta_y,
            x,
            y,
            modifiers,
        }
    }
}

/// Shared UI event payload used by internal host/runtime code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiEvent {
    /// Surface resize.
    Resize(UiResize),
    /// Keyboard input.
    Key(ConsoleKeyEvent),
    /// Mouse input.
    Mouse(ConsoleEvent),
    /// Bracketed paste content.
    Paste(ConsoleEvent),
    /// Focus gained.
    FocusGained(ConsoleEvent),
    /// Focus lost.
    FocusLost(ConsoleEvent),
    /// Graph-only close request.
    CloseRequested,
    /// Graph-only wheel input.
    Wheel(UiWheel),
}

impl UiEvent {
    /// Hint for host loops that a resize usually implies redraw/layout work.
    #[must_use]
    pub fn suggests_request_redraw(&self) -> bool {
        matches!(self, Self::Resize(_))
    }

    /// Projects the shared event into the public `Std.Tui` runtime event model.
    #[must_use]
    pub fn into_tui_event(self) -> Option<TuiEvent> {
        match self {
            Self::Resize(resize) => Some(TuiEvent::Resize {
                old_width: resize.old_width.unwrap_or(0),
                old_height: resize.old_height.unwrap_or(0),
                width: resize.width,
                height: resize.height,
            }),
            Self::Key(key) => Some(TuiEvent::Key(key)),
            Self::Mouse(event) => Some(TuiEvent::Mouse(event)),
            Self::Paste(event) => Some(TuiEvent::Paste(event)),
            Self::FocusGained(event) => Some(TuiEvent::FocusGained(event)),
            Self::FocusLost(event) => Some(TuiEvent::FocusLost(event)),
            Self::CloseRequested | Self::Wheel(_) => None,
        }
    }

    /// Projects the shared event into the public `Std.Graph` runtime event model.
    #[must_use]
    pub fn into_graph_event(self) -> Option<GraphEvent> {
        match self {
            Self::CloseRequested => Some(GraphEvent::CloseRequested),
            Self::Resize(resize) => Some(GraphEvent::Resize {
                width: resize.width,
                height: resize.height,
            }),
            Self::Key(key) => Some(GraphEvent::Key(key)),
            Self::Mouse(event) => Some(GraphEvent::Mouse {
                action: event.mouse_action,
                button: event.mouse_button,
                x: event.mouse_x,
                y: event.mouse_y,
                shift: event.shift,
                ctrl: event.ctrl,
                alt: event.alt,
                meta: event.meta,
            }),
            Self::Wheel(wheel) => Some(GraphEvent::Wheel {
                delta_x: wheel.delta_x,
                delta_y: wheel.delta_y,
                x: wheel.x,
                y: wheel.y,
                shift: wheel.modifiers.shift,
                ctrl: wheel.modifiers.ctrl,
                alt: wheel.modifiers.alt,
                meta: wheel.modifiers.meta,
            }),
            Self::Paste(_) | Self::FocusGained(_) | Self::FocusLost(_) => None,
        }
    }
}

impl From<TuiEvent> for UiEvent {
    fn from(value: TuiEvent) -> Self {
        match value {
            TuiEvent::Resize {
                old_width,
                old_height,
                width,
                height,
            } => Self::Resize(UiResize::new(
                Some(old_width),
                Some(old_height),
                width,
                height,
            )),
            TuiEvent::Key(key) => Self::Key(key),
            TuiEvent::Mouse(event) => Self::Mouse(event),
            TuiEvent::Paste(event) => Self::Paste(event),
            TuiEvent::FocusGained(event) => Self::FocusGained(event),
            TuiEvent::FocusLost(event) => Self::FocusLost(event),
        }
    }
}

impl From<GraphEvent> for UiEvent {
    fn from(value: GraphEvent) -> Self {
        match value {
            GraphEvent::CloseRequested => Self::CloseRequested,
            GraphEvent::Resize { width, height } => {
                Self::Resize(UiResize::new(None, None, width, height))
            }
            GraphEvent::Key(key) => Self::Key(key),
            GraphEvent::Mouse {
                action,
                button,
                x,
                y,
                shift,
                ctrl,
                alt,
                meta,
            } => Self::Mouse(ConsoleEvent::mouse(
                action, button, x, y, shift, ctrl, alt, meta,
            )),
            GraphEvent::Wheel {
                delta_x,
                delta_y,
                x,
                y,
                shift,
                ctrl,
                alt,
                meta,
            } => Self::Wheel(UiWheel::new(
                delta_x,
                delta_y,
                x,
                y,
                UiModifiers::new(shift, ctrl, alt, meta),
            )),
        }
    }
}

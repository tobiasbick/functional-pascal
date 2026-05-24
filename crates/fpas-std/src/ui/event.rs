//! Internal event model shared by terminal and graphics runtimes.
//!
//! This type reduces repeated event reshaping between runtime layers while the
//! public Pascal-facing unit APIs remain unchanged.

use crate::{ConsoleKeyEvent, GraphEvent, TuiEvent};

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

/// Shared mouse payload used by internal UI events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiMouse {
    pub action: usize,
    pub button: usize,
    pub x: i64,
    pub y: i64,
    pub modifiers: UiModifiers,
}

impl UiMouse {
    /// Creates one mouse payload.
    #[must_use]
    pub const fn new(action: usize, button: usize, x: i64, y: i64, modifiers: UiModifiers) -> Self {
        Self {
            action,
            button,
            x,
            y,
            modifiers,
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
    Mouse(UiMouse),
    /// Bracketed paste content.
    Paste(String),
    /// Focus gained.
    FocusGained,
    /// Focus lost.
    FocusLost,
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
        TuiEvent::from_ui_event(self)
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
            Self::Mouse(mouse) => Some(GraphEvent::Mouse {
                action: mouse.action,
                button: mouse.button,
                x: mouse.x,
                y: mouse.y,
                shift: mouse.modifiers.shift,
                ctrl: mouse.modifiers.ctrl,
                alt: mouse.modifiers.alt,
                meta: mouse.modifiers.meta,
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
            Self::Paste(_) | Self::FocusGained | Self::FocusLost => None,
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
            TuiEvent::Mouse(event) => Self::Mouse(UiMouse::new(
                event.mouse_action,
                event.mouse_button,
                event.mouse_x,
                event.mouse_y,
                UiModifiers::new(event.shift, event.ctrl, event.alt, event.meta),
            )),
            TuiEvent::Paste(event) => Self::Paste(event.text),
            TuiEvent::FocusGained(_) => Self::FocusGained,
            TuiEvent::FocusLost(_) => Self::FocusLost,
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
            } => Self::Mouse(UiMouse::new(
                action,
                button,
                x,
                y,
                UiModifiers::new(shift, ctrl, alt, meta),
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

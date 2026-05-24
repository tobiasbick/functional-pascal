//! Internal event model shared by terminal and graphics runtimes.
//!
//! This type reduces repeated event reshaping between runtime layers while the
//! public Pascal-facing unit APIs remain unchanged.

use crate::ConsoleKeyEvent;

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
}

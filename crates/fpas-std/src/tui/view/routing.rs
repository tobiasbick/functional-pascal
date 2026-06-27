//! Typed retained-view event routes and outcomes.

use crate::{CommandEvent, ConsoleKeyEvent, DamageRegion, UiMouse, UiWheel};

use super::{ViewId, ViewRegistry};

/// Event payload routed through a retained view path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutedEvent {
    /// Keyboard event routed from the focused leaf.
    Key(ConsoleKeyEvent),
    /// Pointer event routed from capture or hit-testing.
    Mouse(UiMouse),
    /// Wheel event routed from capture or hit-testing.
    Wheel(UiWheel),
    /// Paste routed from the focused leaf.
    Paste(String),
    /// Terminal focus entered the application.
    FocusGained,
    /// Terminal focus left the application.
    FocusLost,
}

/// One stage of retained event propagation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPhase {
    /// Root-to-parent pre-processing.
    Capture,
    /// Delivery to the target view.
    Target,
    /// Parent-to-root post-processing.
    Bubble,
    /// Application fallback after the route remains unconsumed.
    Default,
}

/// Result produced by a retained event handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventOutcome {
    /// Continue routing.
    Ignored,
    /// Stop routing without another action.
    Consumed,
    /// Dispatch a sourced command.
    Command(CommandEvent),
    /// Move keyboard focus to the supplied view.
    RequestFocus(ViewId),
    /// Capture subsequent pointer events for the supplied view.
    CapturePointer(ViewId),
    /// Release the current pointer capture.
    ReleasePointer,
    /// Invalidate a retained screen region.
    RequestRedraw(DamageRegion),
}

/// Target and propagation paths resolved for one event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRoute {
    /// Event being routed.
    pub event: RoutedEvent,
    /// Target leaf, when the event has one.
    pub target: Option<ViewId>,
    /// Ancestors ordered root-to-parent for capture.
    pub capture: Vec<ViewId>,
    /// Ancestors ordered parent-to-root for bubbling.
    pub bubble: Vec<ViewId>,
}

impl ViewRegistry {
    /// Capture all subsequent pointer events for `id` until explicitly released or removed.
    pub fn capture_pointer(&mut self, id: ViewId) -> bool {
        if self.resolved(id).is_some_and(|view| view.state.enabled) {
            self.pointer_capture = Some(id);
            true
        } else {
            false
        }
    }

    /// Release any active pointer capture.
    pub fn release_pointer(&mut self) {
        self.pointer_capture = None;
    }

    /// Begin a pointer press owned by `id` and capture subsequent pointer events.
    pub fn begin_pointer_press(&mut self, id: ViewId) -> bool {
        if self.capture_pointer(id) {
            self.pointer_press = Some(id);
            true
        } else {
            false
        }
    }

    /// End the active pointer press and release pointer capture.
    pub fn end_pointer_press(&mut self) -> Option<ViewId> {
        let pressed = self.pointer_press.take();
        self.release_pointer();
        pressed
    }

    /// Return the view that owns the active pointer press.
    #[must_use]
    pub fn pressed_pointer(&self) -> Option<ViewId> {
        self.pointer_press
    }

    /// Cancel all in-flight pointer-owned state.
    ///
    /// This is stronger than [`Self::release_pointer`]: it also drops frame move/resize and frame
    /// scroll-thumb drags that depend on capture continuing.
    pub fn cancel_pointer_interactions(&mut self) -> bool {
        let changed = self.pointer_capture.is_some()
            || self.pointer_press.is_some()
            || self.window_interaction.is_some()
            || self.frame_scroll_interaction.is_some();
        self.pointer_capture = None;
        self.pointer_press = None;
        self.window_interaction = None;
        self.frame_scroll_interaction = None;
        changed
    }

    /// Return the view that currently owns pointer capture.
    #[must_use]
    pub fn captured_pointer(&self) -> Option<ViewId> {
        self.pointer_capture
    }

    /// Resolve a pointer target from capture first, then clipped hit-testing.
    #[must_use]
    pub fn pointer_target(
        &self,
        mouse_x: i64,
        mouse_y: i64,
        scope: Option<&[ViewId]>,
    ) -> Option<ViewId> {
        if let Some(captured) = self.pointer_capture
            && scope.is_none_or(|ids| ids.contains(&captured))
            && self
                .resolved(captured)
                .is_some_and(|view| view.state.enabled)
        {
            return Some(captured);
        }

        self.topmost_view_at(mouse_x.saturating_sub(1), mouse_y.saturating_sub(1), scope)
            .filter(|id| self.state(*id).is_some_and(|state| state.enabled))
    }

    /// Resolve an event target and its capture/bubble paths.
    #[must_use]
    pub fn route_event(&self, event: RoutedEvent, scope: Option<&[ViewId]>) -> EventRoute {
        let target = match &event {
            RoutedEvent::Key(_) | RoutedEvent::Paste(_) => self
                .focused_id()
                .filter(|id| scope.is_none_or(|ids| ids.contains(id))),
            RoutedEvent::Mouse(mouse) => self.pointer_target(mouse.x, mouse.y, scope),
            RoutedEvent::Wheel(wheel) => self.pointer_target(wheel.x, wheel.y, scope),
            RoutedEvent::FocusGained | RoutedEvent::FocusLost => None,
        };

        let ancestors = target
            .map(|id| self.ancestors_inclusive(id))
            .unwrap_or_default();
        let bubble = ancestors
            .iter()
            .skip(1)
            .copied()
            .filter(|id| {
                self.options(*id)
                    .is_some_and(|options| options.post_process)
            })
            .collect();
        let capture = ancestors
            .iter()
            .skip(1)
            .rev()
            .copied()
            .filter(|id| self.options(*id).is_some_and(|options| options.pre_process))
            .collect();

        EventRoute {
            event,
            target,
            capture,
            bubble,
        }
    }

    /// Apply a stateful routing outcome owned by the view registry.
    ///
    /// Returns `true` when registry state changed. Command and redraw outcomes are returned to the
    /// host layer and therefore do not mutate the registry here.
    pub fn apply_event_outcome(&mut self, outcome: &EventOutcome) -> bool {
        match *outcome {
            EventOutcome::RequestFocus(id) => self.focus_view(id).0,
            EventOutcome::CapturePointer(id) => self.capture_pointer(id),
            EventOutcome::ReleasePointer => {
                let changed = self.pointer_capture.is_some();
                self.release_pointer();
                changed
            }
            EventOutcome::Ignored
            | EventOutcome::Consumed
            | EventOutcome::Command(_)
            | EventOutcome::RequestRedraw(_) => false,
        }
    }
}

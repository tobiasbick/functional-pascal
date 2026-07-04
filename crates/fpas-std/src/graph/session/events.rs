//! Hosted graph redraw and event queue state.
//!
//! **Documentation:** `docs/pascal/std/graph/session.md`

use super::GraphSession;
use crate::error::StdError;
use crate::graph::backend;
use crate::graph::event::GraphEvent;
use crate::ui::{UiEvent, UiResize};
use fpas_bytecode::SourceLocation;

impl GraphSession {
    /// Marks the active session as needing a hosted redraw.
    pub fn request_redraw(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.ensure_open(
            "Application.RequestRedraw(App) requires an open graphics session.",
            "Open the application before requesting a redraw.",
            location,
        )?;
        self.redraw_pending = true;
        Ok(())
    }

    /// Marks the active session as needing a hosted redraw when none is already pending.
    pub fn request_redraw_if_absent(&mut self, location: SourceLocation) -> Result<(), StdError> {
        self.ensure_open(
            "Application.RequestRedraw(App) requires an open graphics session.",
            "Open the application before requesting a redraw.",
            location,
        )?;
        if !self.redraw_pending {
            self.redraw_pending = true;
        }
        Ok(())
    }

    /// Returns whether a hosted redraw is pending without consuming it.
    pub fn peek_redraw_pending(&self, location: SourceLocation) -> Result<bool, StdError> {
        self.ensure_open(
            "Hosted graph redraw requires an open graphics session.",
            "Open the application before querying redraw state.",
            location,
        )?;
        Ok(self.redraw_pending)
    }

    /// Consumes and returns whether a hosted redraw was pending.
    pub fn take_redraw_pending(&mut self, location: SourceLocation) -> Result<bool, StdError> {
        self.ensure_open(
            "Hosted graph redraw requires an open graphics session.",
            "Open the application before consuming redraw state.",
            location,
        )?;
        let pending = self.redraw_pending;
        self.redraw_pending = false;
        Ok(pending)
    }

    /// Waits up to `timeout_ms` for the next hosted UI event from the native backend.
    pub fn read_host_ui_event_timeout(
        &mut self,
        timeout_ms: i64,
        location: SourceLocation,
    ) -> Result<Option<UiEvent>, StdError> {
        self.ensure_open(
            "Std.Graph hosted event wait requires an open graphics session.",
            "Open the application before waiting for events.",
            location,
        )?;

        if let Some(event) = self.pending_events.pop_front() {
            self.apply_polled_event(&event, location)?;
            return Ok(Some(event));
        }

        let event = backend::read_graph_event_timeout(timeout_ms, location)?;
        if let Some(event) = &event {
            self.apply_polled_event(event, location)?;
        }
        Ok(event)
    }

    /// Queues one normalized host event for the active session.
    pub fn push_event(
        &mut self,
        event: GraphEvent,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        self.ensure_open(
            "Std.Graph host event injection requires an open graphics session.",
            "Open the application before queueing host events for it.",
            location,
        )?;

        self.pending_events.push_back(event.into_ui_event());
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn push_event_for_tests(&mut self, event: GraphEvent) {
        self.pending_events.push_back(event.into_ui_event());
    }

    fn apply_polled_event(
        &mut self,
        event: &UiEvent,
        location: SourceLocation,
    ) -> Result<(), StdError> {
        if let UiEvent::Resize(UiResize { width, height, .. }) = event {
            self.width = *width;
            self.height = *height;
            self.backbuffer.resize(*width, *height, location)?;
        }
        Ok(())
    }
}

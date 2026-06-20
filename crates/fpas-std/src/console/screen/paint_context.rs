//! Hard clipping and state restoration for retained-view paint callbacks.

use super::{ConsoleState, PaintContext, WindowRect};
use crate::ViewRect;

impl ConsoleState {
    pub(in super::super) fn begin_view_paint(&mut self, rect: ViewRect, clip: ViewRect) -> bool {
        if self.paint_context.is_some() {
            return false;
        }
        let Some(view_window) = WindowRect::from_zero_based_rect(
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            self.width,
            self.height,
        ) else {
            return false;
        };
        let Some(clip_window) = WindowRect::from_zero_based_rect(
            clip.x,
            clip.y,
            clip.width,
            clip.height,
            self.width,
            self.height,
        ) else {
            return false;
        };

        self.paint_context = Some(PaintContext {
            clip: clip_window,
            saved_window: self.window,
            saved_cursor_x: self.cursor_x,
            saved_cursor_y: self.cursor_y,
            saved_pending_wrap: self.pending_wrap,
        });
        self.window = view_window;
        self.cursor_x = 1;
        self.cursor_y = 1;
        self.pending_wrap = false;
        true
    }

    pub(in super::super) fn end_view_paint(&mut self) {
        let Some(context) = self.paint_context.take() else {
            return;
        };
        self.window = context.saved_window;
        self.cursor_x = context.saved_cursor_x;
        self.cursor_y = context.saved_cursor_y;
        self.pending_wrap = context.saved_pending_wrap;
    }

    pub(super) fn clip_window(&self, window: WindowRect) -> Option<WindowRect> {
        match self.paint_context {
            Some(context) => window.intersection(context.clip),
            None => Some(window),
        }
    }

    pub(super) fn can_paint_cell(&self, x: u16, y: u16) -> bool {
        self.paint_context
            .is_none_or(|context| context.clip.contains(x, y))
    }
}

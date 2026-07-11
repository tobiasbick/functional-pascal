use crate::console::Console;
use crate::console_event::event_kind_index;
use crate::{ConsoleEvent, ConsoleKeyEvent, UiEvent, UiModifiers, UiMouse, UiResize};

/// Host-normalized event emitted by an active TUI application session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiEvent {
    /// Key input mapped from the console event stream.
    Key(ConsoleKeyEvent),
    /// Terminal resize with both the old and new surface sizes.
    Resize {
        old_width: i64,
        old_height: i64,
        width: i64,
        height: i64,
    },
    /// Mouse input preserved from the console event stream.
    Mouse(UiMouse),
    /// Bracketed-paste content; best-effort on terminals that support it.
    Paste(String),
    /// Terminal focus gained; best-effort / optional on many terminals.
    FocusGained,
    /// Terminal focus lost; best-effort / optional on many terminals.
    FocusLost,
}

/// Maps one console event into the shared internal UI event model.
pub(super) fn map_console_ui_event(console: &mut Console, event: ConsoleEvent) -> Option<UiEvent> {
    if event.kind == event_kind_index("Resize") {
        let (Ok(width), Ok(height)) = (u16::try_from(event.width), u16::try_from(event.height))
        else {
            return None;
        };
        if width == 0 || height == 0 {
            return None;
        }

        let old_width = console.screen_width();
        let old_height = console.screen_height();
        console.resize(width, height);
        return Some(UiEvent::Resize(UiResize::new(
            Some(old_width),
            Some(old_height),
            event.width,
            event.height,
        )));
    }

    if event.kind == event_kind_index("Key") {
        return Some(UiEvent::Key(event.key));
    }

    if event.kind == event_kind_index("Mouse") {
        return Some(UiEvent::Mouse(UiMouse::new(
            event.mouse_action,
            event.mouse_button,
            event.mouse_x,
            event.mouse_y,
            UiModifiers::new(event.shift, event.ctrl, event.alt, event.meta),
        )));
    }

    if event.kind == event_kind_index("Paste") {
        return Some(UiEvent::Paste(event.text));
    }

    if event.kind == event_kind_index("FocusGained") {
        return Some(UiEvent::FocusGained);
    }

    if event.kind == event_kind_index("FocusLost") {
        return Some(UiEvent::FocusLost);
    }

    None
}

/// Maps one console event into the internal `TuiEvent` model.
pub(crate) fn map_console_event(console: &mut Console, event: ConsoleEvent) -> Option<TuiEvent> {
    map_console_ui_event(console, event).and_then(ui_event_as_tui_event)
}

fn ui_event_as_tui_event(value: UiEvent) -> Option<TuiEvent> {
    match value {
        UiEvent::Resize(resize) => Some(TuiEvent::Resize {
            old_width: resize.old_width.unwrap_or(0),
            old_height: resize.old_height.unwrap_or(0),
            width: resize.width,
            height: resize.height,
        }),
        UiEvent::Key(key) => Some(TuiEvent::Key(key)),
        UiEvent::Mouse(mouse) => Some(TuiEvent::Mouse(mouse)),
        UiEvent::Paste(text) => Some(TuiEvent::Paste(text)),
        UiEvent::FocusGained => Some(TuiEvent::FocusGained),
        UiEvent::FocusLost => Some(TuiEvent::FocusLost),
        UiEvent::CloseRequested | UiEvent::Wheel(_) => None,
    }
}

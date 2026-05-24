use crate::console::Console;
use crate::console_event::event_kind_index;
use crate::{ConsoleEvent, ConsoleKeyEvent, UiEvent, UiMouse};

/// Variants for `Std.Tui.EventKind`.
pub const TUI_EVENT_KIND_VARIANTS: &[&str] = &["Key", "Resize", "Mouse"];

/// Variants for `Std.Tui.ExitReason` used by hosted dispatch and `Application.Run`.
pub const TUI_EXIT_REASON_VARIANTS: &[&str] =
    &["UserQuit", "HostStop", "HostAndUserStop", "HostShutdown"];

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
    Mouse(ConsoleEvent),
    /// Bracketed-paste content; best-effort on terminals that support it.
    Paste(ConsoleEvent),
    /// Terminal focus gained; best-effort / optional on many terminals.
    FocusGained(ConsoleEvent),
    /// Terminal focus lost; best-effort / optional on many terminals.
    FocusLost(ConsoleEvent),
}

impl TuiEvent {
    /// Projects one internal shared UI event into the public `Std.Tui` event model.
    #[must_use]
    pub(crate) fn from_ui_event(value: UiEvent) -> Option<Self> {
        match value {
            UiEvent::Resize(resize) => Some(Self::Resize {
                old_width: resize.old_width.unwrap_or(0),
                old_height: resize.old_height.unwrap_or(0),
                width: resize.width,
                height: resize.height,
            }),
            UiEvent::Key(key) => Some(Self::Key(key)),
            UiEvent::Mouse(mouse) => Some(Self::Mouse(console_event_from_ui_mouse(mouse))),
            UiEvent::Paste(text) => Some(Self::Paste(ConsoleEvent::paste(text))),
            UiEvent::FocusGained => Some(Self::FocusGained(ConsoleEvent::focus_gained())),
            UiEvent::FocusLost => Some(Self::FocusLost(ConsoleEvent::focus_lost())),
            UiEvent::CloseRequested | UiEvent::Wheel(_) => None,
        }
    }
}

fn console_event_from_ui_mouse(mouse: UiMouse) -> ConsoleEvent {
    ConsoleEvent::mouse(
        mouse.action,
        mouse.button,
        mouse.x,
        mouse.y,
        mouse.modifiers.shift,
        mouse.modifiers.ctrl,
        mouse.modifiers.alt,
        mouse.modifiers.meta,
    )
}

pub(crate) fn map_console_event(console: &mut Console, event: ConsoleEvent) -> Option<TuiEvent> {
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
        return Some(TuiEvent::Resize {
            old_width,
            old_height,
            width: event.width,
            height: event.height,
        });
    }

    if event.kind == event_kind_index("Key") {
        return Some(TuiEvent::Key(event.key));
    }

    if event.kind == event_kind_index("Mouse") {
        return Some(TuiEvent::Mouse(event));
    }

    if event.kind == event_kind_index("Paste") {
        return Some(TuiEvent::Paste(event));
    }

    if event.kind == event_kind_index("FocusGained") {
        return Some(TuiEvent::FocusGained(event));
    }

    if event.kind == event_kind_index("FocusLost") {
        return Some(TuiEvent::FocusLost(event));
    }

    None
}

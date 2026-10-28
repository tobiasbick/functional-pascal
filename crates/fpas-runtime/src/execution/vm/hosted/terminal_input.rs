//! Terminal input shared by ordinary execution and debugger hosts.

use crate::stdlib::{Console, KeyInput};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

/// Key names transported by terminal hosts and input scripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugTerminalKeyKind {
    /// Escape.
    Escape,
    /// Horizontal tab.
    Tab,
    /// Enter or return.
    Enter,
    /// Backspace.
    Backspace,
    /// Space.
    Space,
    /// Cursor up.
    Up,
    /// Cursor down.
    Down,
    /// Cursor left.
    Left,
    /// Cursor right.
    Right,
    /// Home.
    Home,
    /// End.
    End,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Insert.
    Insert,
    /// Delete.
    Delete,
    /// Function key F1 through F12.
    Function(u8),
    /// One Unicode character.
    Character(char),
}

impl DebugTerminalKeyKind {
    /// Parse a named terminal key, including F1 through F12.
    /// Character input carries its scalar separately and returns `None` here.
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        match name {
            "Escape" => Some(Self::Escape),
            "Tab" => Some(Self::Tab),
            "Enter" => Some(Self::Enter),
            "Backspace" => Some(Self::Backspace),
            "Space" => Some(Self::Space),
            "Up" => Some(Self::Up),
            "Down" => Some(Self::Down),
            "Left" => Some(Self::Left),
            "Right" => Some(Self::Right),
            "Home" => Some(Self::Home),
            "End" => Some(Self::End),
            "PageUp" => Some(Self::PageUp),
            "PageDown" => Some(Self::PageDown),
            "Insert" => Some(Self::Insert),
            "Delete" => Some(Self::Delete),
            name => name
                .strip_prefix('F')
                .and_then(|number| number.parse::<u8>().ok())
                .filter(|number| (1..=12).contains(number))
                .map(Self::Function),
        }
    }
}

/// One terminal key and its modifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugTerminalKeyEvent {
    /// Logical key.
    pub kind: DebugTerminalKeyKind,
    /// Whether Shift is active.
    pub shift: bool,
    /// Whether Control is active.
    pub ctrl: bool,
    /// Whether Alt is active.
    pub alt: bool,
    /// Whether Meta is active.
    pub meta: bool,
}

/// Mouse actions transported by terminal hosts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugTerminalMouseAction {
    /// Button press.
    Down,
    /// Button release.
    Up,
    /// Drag with a pressed button.
    Drag,
    /// Pointer movement.
    Move,
    /// Wheel down.
    ScrollDown,
    /// Wheel up.
    ScrollUp,
    /// Wheel left.
    ScrollLeft,
    /// Wheel right.
    ScrollRight,
}

/// Mouse buttons transported by terminal hosts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugTerminalMouseButton {
    /// No button.
    None,
    /// Left button.
    Left,
    /// Right button.
    Right,
    /// Middle button.
    Middle,
}

/// One terminal event delivered to an executing program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugTerminalEvent {
    /// Keyboard input.
    Key(DebugTerminalKeyEvent),
    /// Mouse input with one-based cell coordinates.
    Mouse {
        /// Mouse action.
        action: DebugTerminalMouseAction,
        /// Mouse button.
        button: DebugTerminalMouseButton,
        /// One-based column.
        x: u16,
        /// One-based row.
        y: u16,
        /// Whether Shift is active.
        shift: bool,
        /// Whether Control is active.
        ctrl: bool,
        /// Whether Alt is active.
        alt: bool,
        /// Whether Meta is active.
        meta: bool,
    },
    /// Terminal dimensions in cells.
    Resize {
        /// Width in cells.
        width: u16,
        /// Height in cells.
        height: u16,
    },
    /// Bracketed paste text.
    Paste(String),
    /// Terminal focus gained.
    FocusGained,
    /// Terminal focus lost.
    FocusLost,
}

/// Queue one host event, keeping terminal dimensions and input APIs consistent.
pub(crate) fn queue_event(console: &mut Console, input: &mut KeyInput, event: &DebugTerminalEvent) {
    match event {
        DebugTerminalEvent::Key(event) => {
            input.push_host_event(Event::Key(KeyEvent::new(
                terminal_key(event.kind),
                terminal_modifiers(event.shift, event.ctrl, event.alt, event.meta),
            )));
        }
        DebugTerminalEvent::Mouse {
            action,
            button,
            x,
            y,
            shift,
            ctrl,
            alt,
            meta,
        } => {
            input.push_host_event(Event::Mouse(MouseEvent {
                kind: terminal_mouse(*action, *button),
                column: x.saturating_sub(1),
                row: y.saturating_sub(1),
                modifiers: terminal_modifiers(*shift, *ctrl, *alt, *meta),
            }));
        }
        DebugTerminalEvent::Resize { width, height } => {
            console.resize(*width, *height);
            input.push_host_event(Event::Resize(*width, *height));
        }
        DebugTerminalEvent::Paste(text) => {
            input.push_host_event(Event::Paste(text.clone()));
        }
        DebugTerminalEvent::FocusGained => {
            input.push_host_event(Event::FocusGained);
        }
        DebugTerminalEvent::FocusLost => {
            input.push_host_event(Event::FocusLost);
        }
    }
}

fn terminal_key(kind: DebugTerminalKeyKind) -> KeyCode {
    match kind {
        DebugTerminalKeyKind::Escape => KeyCode::Esc,
        DebugTerminalKeyKind::Tab => KeyCode::Tab,
        DebugTerminalKeyKind::Enter => KeyCode::Enter,
        DebugTerminalKeyKind::Backspace => KeyCode::Backspace,
        DebugTerminalKeyKind::Space => KeyCode::Char(' '),
        DebugTerminalKeyKind::Up => KeyCode::Up,
        DebugTerminalKeyKind::Down => KeyCode::Down,
        DebugTerminalKeyKind::Left => KeyCode::Left,
        DebugTerminalKeyKind::Right => KeyCode::Right,
        DebugTerminalKeyKind::Home => KeyCode::Home,
        DebugTerminalKeyKind::End => KeyCode::End,
        DebugTerminalKeyKind::PageUp => KeyCode::PageUp,
        DebugTerminalKeyKind::PageDown => KeyCode::PageDown,
        DebugTerminalKeyKind::Insert => KeyCode::Insert,
        DebugTerminalKeyKind::Delete => KeyCode::Delete,
        DebugTerminalKeyKind::Function(value) => KeyCode::F(value),
        DebugTerminalKeyKind::Character(character) => KeyCode::Char(character),
    }
}

fn terminal_mouse(
    action: DebugTerminalMouseAction,
    button: DebugTerminalMouseButton,
) -> MouseEventKind {
    match action {
        DebugTerminalMouseAction::Down => MouseEventKind::Down(terminal_button(button)),
        DebugTerminalMouseAction::Up => MouseEventKind::Up(terminal_button(button)),
        DebugTerminalMouseAction::Drag => MouseEventKind::Drag(terminal_button(button)),
        DebugTerminalMouseAction::Move => MouseEventKind::Moved,
        DebugTerminalMouseAction::ScrollDown => MouseEventKind::ScrollDown,
        DebugTerminalMouseAction::ScrollUp => MouseEventKind::ScrollUp,
        DebugTerminalMouseAction::ScrollLeft => MouseEventKind::ScrollLeft,
        DebugTerminalMouseAction::ScrollRight => MouseEventKind::ScrollRight,
    }
}

fn terminal_button(button: DebugTerminalMouseButton) -> MouseButton {
    match button {
        DebugTerminalMouseButton::Left => MouseButton::Left,
        DebugTerminalMouseButton::Right => MouseButton::Right,
        DebugTerminalMouseButton::Middle => MouseButton::Middle,
        DebugTerminalMouseButton::None => MouseButton::Left,
    }
}

fn terminal_modifiers(shift: bool, ctrl: bool, alt: bool, meta: bool) -> KeyModifiers {
    let mut modifiers = KeyModifiers::NONE;
    if shift {
        modifiers.insert(KeyModifiers::SHIFT);
    }
    if ctrl {
        modifiers.insert(KeyModifiers::CONTROL);
    }
    if alt {
        modifiers.insert(KeyModifiers::ALT);
    }
    if meta {
        modifiers.insert(KeyModifiers::SUPER);
    }
    modifiers
}

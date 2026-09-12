//! Thread-safe terminal bridge shared by the debugger actor and protocol host.

use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use fpas_std::{Console, KeyInput};

use super::DebuggeeChannel;
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError, DebuggeeInputResult};

/// Key names transported by debugger terminal clients.
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

/// Mouse actions transported by debugger terminal clients.
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

/// Mouse buttons transported by debugger terminal clients.
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

/// One terminal event delivered asynchronously to a debuggee.
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

#[derive(Default)]
struct TerminalOutput {
    pending: Mutex<Vec<u8>>,
    total_bytes: AtomicUsize,
}

struct TerminalWriter {
    output: Arc<TerminalOutput>,
}

impl Write for TerminalWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.output
            .pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .extend_from_slice(bytes);
        self.output
            .total_bytes
            .fetch_add(bytes.len(), Ordering::Relaxed);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Cloneable debugger-side access to terminal output and input queues.
#[derive(Clone)]
pub struct DebugTerminalHandle {
    output: Arc<TerminalOutput>,
    console: Arc<Mutex<Console>>,
    key_input: Arc<Mutex<KeyInput>>,
    debuggee: Arc<Mutex<DebuggeeChannel>>,
}

impl DebugTerminalHandle {
    pub(in crate::vm::debug) fn create(
        debuggee: Arc<Mutex<DebuggeeChannel>>,
    ) -> (Self, Arc<Mutex<Console>>, Arc<Mutex<KeyInput>>) {
        let output = Arc::new(TerminalOutput::default());
        let console = Arc::new(Mutex::new(Console::with_capturing_writer(Box::new(
            TerminalWriter {
                output: Arc::clone(&output),
            },
        ))));
        let key_input = Arc::new(Mutex::new(KeyInput::without_os_events()));
        (
            Self {
                output,
                console: Arc::clone(&console),
                key_input: Arc::clone(&key_input),
                debuggee,
            },
            console,
            key_input,
        )
    }

    /// Remove and return terminal bytes produced since the previous drain.
    #[must_use]
    pub fn take_output(&self) -> Vec<u8> {
        std::mem::take(
            &mut *self
                .output
                .pending
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }

    /// Return the total number of terminal bytes emitted by this session.
    #[must_use]
    pub fn output_byte_count(&self) -> usize {
        self.output.total_bytes.load(Ordering::Relaxed)
    }

    /// Queue terminal events without requiring ownership of the running debug session.
    ///
    /// # Errors
    ///
    /// Returns a debugger input error after EOF, disconnect, or the configured input limit.
    pub fn push_events(
        &self,
        events: &[DebugTerminalEvent],
    ) -> Result<DebuggeeInputResult, DebugSessionError> {
        let bytes = events.iter().map(event_size).sum::<usize>();
        let session_bytes = self
            .debuggee
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .accept_input(bytes)
            .map_err(terminal_input_error)?;
        let mut console = self
            .console
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut input = self
            .key_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for event in events {
            queue_event(&mut console, &mut input, event);
        }
        Ok(DebuggeeInputResult {
            bytes,
            session_bytes,
        })
    }
}

fn queue_event(console: &mut Console, input: &mut KeyInput, event: &DebugTerminalEvent) {
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

fn event_size(event: &DebugTerminalEvent) -> usize {
    match event {
        DebugTerminalEvent::Paste(text) => text.len(),
        DebugTerminalEvent::Key(DebugTerminalKeyEvent {
            kind: DebugTerminalKeyKind::Character(character),
            ..
        }) => character.len_utf8(),
        _ => 1,
    }
}

fn terminal_input_error(kind: DebugErrorKind) -> DebugSessionError {
    match kind {
        DebugErrorKind::DebuggeeInputLimit => DebugSessionError {
            kind,
            message: "debuggee terminal input exceeds the session input limit".to_string(),
            hint: "Send fewer terminal events, or raise the debugger input limit.".to_string(),
        },
        DebugErrorKind::DebuggeeInputClosed => DebugSessionError {
            kind,
            message: "debuggee terminal input is closed".to_string(),
            hint: "Do not send terminal events after EOF or disconnect.".to_string(),
        },
        _ => DebugSessionError {
            kind: DebugErrorKind::InvalidState,
            message: "debuggee terminal input is unavailable".to_string(),
            hint: "Send terminal input only while the debug session is connected.".to_string(),
        },
    }
}

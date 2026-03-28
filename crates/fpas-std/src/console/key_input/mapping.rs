use crate::console::key_input::LiveConsoleEvent;
use crate::console_event::{ConsoleEvent, mouse_action_index, mouse_button_index};
use crate::key_event::{ConsoleKeyEvent, key_kind_index};
use crossterm::event::{
    KeyCode, KeyEvent as CrosstermKeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use std::collections::VecDeque;

pub(super) fn map_crossterm_key(key: &CrosstermKeyEvent) -> ConsoleKeyEvent {
    let modifiers = key.modifiers;
    let shift = modifiers.contains(KeyModifiers::SHIFT);
    let ctrl = modifiers.contains(KeyModifiers::CONTROL);
    let alt = modifiers.contains(KeyModifiers::ALT);
    let meta = modifiers.contains(KeyModifiers::SUPER);
    match key.code {
        KeyCode::Char(c) => {
            let kind = if c == ' ' {
                key_kind_index("Space")
            } else {
                key_kind_index("Character")
            };
            ConsoleKeyEvent::new(kind, c, shift, ctrl, alt, meta)
        }
        KeyCode::Enter => {
            ConsoleKeyEvent::new(key_kind_index("Enter"), '\0', shift, ctrl, alt, meta)
        }
        KeyCode::Tab => ConsoleKeyEvent::new(key_kind_index("Tab"), '\0', shift, ctrl, alt, meta),
        KeyCode::Backspace => {
            ConsoleKeyEvent::new(key_kind_index("Backspace"), '\0', shift, ctrl, alt, meta)
        }
        KeyCode::Esc => {
            ConsoleKeyEvent::new(key_kind_index("Escape"), '\0', shift, ctrl, alt, meta)
        }
        KeyCode::Up => ConsoleKeyEvent::new(key_kind_index("Up"), '\0', shift, ctrl, alt, meta),
        KeyCode::Down => ConsoleKeyEvent::new(key_kind_index("Down"), '\0', shift, ctrl, alt, meta),
        KeyCode::Left => ConsoleKeyEvent::new(key_kind_index("Left"), '\0', shift, ctrl, alt, meta),
        KeyCode::Right => {
            ConsoleKeyEvent::new(key_kind_index("Right"), '\0', shift, ctrl, alt, meta)
        }
        KeyCode::Home => ConsoleKeyEvent::new(key_kind_index("Home"), '\0', shift, ctrl, alt, meta),
        KeyCode::End => ConsoleKeyEvent::new(key_kind_index("End"), '\0', shift, ctrl, alt, meta),
        KeyCode::PageUp => {
            ConsoleKeyEvent::new(key_kind_index("PageUp"), '\0', shift, ctrl, alt, meta)
        }
        KeyCode::PageDown => {
            ConsoleKeyEvent::new(key_kind_index("PageDown"), '\0', shift, ctrl, alt, meta)
        }
        KeyCode::Insert => {
            ConsoleKeyEvent::new(key_kind_index("Insert"), '\0', shift, ctrl, alt, meta)
        }
        KeyCode::Delete => {
            ConsoleKeyEvent::new(key_kind_index("Delete"), '\0', shift, ctrl, alt, meta)
        }
        KeyCode::F(n) if (1..=12).contains(&n) => ConsoleKeyEvent::new(
            key_kind_index(&format!("F{n}")),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        ),
        _ => ConsoleKeyEvent::new(0, '\0', shift, ctrl, alt, meta),
    }
}

pub(super) fn map_key_for_read(code: KeyCode, pending: &mut VecDeque<char>) -> char {
    match code {
        KeyCode::Char(c) => c,
        KeyCode::Enter => '\r',
        KeyCode::Tab => '\t',
        KeyCode::Backspace => '\x08',
        KeyCode::Esc => '\x1B',
        KeyCode::Up
        | KeyCode::Down
        | KeyCode::Left
        | KeyCode::Right
        | KeyCode::Home
        | KeyCode::End
        | KeyCode::PageUp
        | KeyCode::PageDown
        | KeyCode::Insert
        | KeyCode::Delete
        | KeyCode::F(_) => {
            pending.push_back(extended_scan(code));
            '\0'
        }
        _ => '\0',
    }
}

pub(super) fn map_console_event(event: LiveConsoleEvent) -> ConsoleEvent {
    match event {
        LiveConsoleEvent::Key(key) => ConsoleEvent::key(map_crossterm_key(&key)),
        LiveConsoleEvent::Mouse(mouse) => map_mouse_event(mouse),
        LiveConsoleEvent::Resize(width, height) => {
            ConsoleEvent::resize(i64::from(width), i64::from(height))
        }
        LiveConsoleEvent::Paste(text) => ConsoleEvent::paste(text),
        LiveConsoleEvent::FocusGained => ConsoleEvent::focus_gained(),
        LiveConsoleEvent::FocusLost => ConsoleEvent::focus_lost(),
    }
}

fn map_mouse_event(mouse: MouseEvent) -> ConsoleEvent {
    let shift = mouse.modifiers.contains(KeyModifiers::SHIFT);
    let ctrl = mouse.modifiers.contains(KeyModifiers::CONTROL);
    let alt = mouse.modifiers.contains(KeyModifiers::ALT);
    let meta = mouse.modifiers.contains(KeyModifiers::SUPER);
    let action = match mouse.kind {
        MouseEventKind::Down(_) => mouse_action_index("Down"),
        MouseEventKind::Up(_) => mouse_action_index("Up"),
        MouseEventKind::Drag(_) => mouse_action_index("Drag"),
        MouseEventKind::Moved => mouse_action_index("Move"),
        MouseEventKind::ScrollDown => mouse_action_index("ScrollDown"),
        MouseEventKind::ScrollUp => mouse_action_index("ScrollUp"),
        MouseEventKind::ScrollLeft => mouse_action_index("ScrollLeft"),
        MouseEventKind::ScrollRight => mouse_action_index("ScrollRight"),
    };
    let button = match mouse.kind {
        MouseEventKind::Down(button)
        | MouseEventKind::Up(button)
        | MouseEventKind::Drag(button) => map_mouse_button(button),
        MouseEventKind::Moved
        | MouseEventKind::ScrollDown
        | MouseEventKind::ScrollUp
        | MouseEventKind::ScrollLeft
        | MouseEventKind::ScrollRight => mouse_button_index("None"),
    };
    ConsoleEvent::mouse(
        action,
        button,
        i64::from(mouse.column) + 1,
        i64::from(mouse.row) + 1,
        shift,
        ctrl,
        alt,
        meta,
    )
}

fn map_mouse_button(button: MouseButton) -> usize {
    match button {
        MouseButton::Left => mouse_button_index("Left"),
        MouseButton::Right => mouse_button_index("Right"),
        MouseButton::Middle => mouse_button_index("Middle"),
    }
}

/// Turbo Pascal-style extended second byte for arrow keys (scan codes).
fn extended_scan(code: KeyCode) -> char {
    let byte: u8 = match code {
        KeyCode::Up => 72,
        KeyCode::Down => 80,
        KeyCode::Left => 75,
        KeyCode::Right => 77,
        KeyCode::Home => 71,
        KeyCode::End => 79,
        KeyCode::PageUp => 73,
        KeyCode::PageDown => 81,
        KeyCode::Insert => 82,
        KeyCode::Delete => 83,
        KeyCode::F(1) => 59,
        KeyCode::F(2) => 60,
        KeyCode::F(3) => 61,
        KeyCode::F(4) => 62,
        KeyCode::F(5) => 63,
        KeyCode::F(6) => 64,
        KeyCode::F(7) => 65,
        KeyCode::F(8) => 66,
        KeyCode::F(9) => 67,
        KeyCode::F(10) => 68,
        KeyCode::F(11) => 133,
        KeyCode::F(12) => 134,
        _ => 255,
    };
    char::from_u32(u32::from(byte)).unwrap_or('\u{00FF}')
}

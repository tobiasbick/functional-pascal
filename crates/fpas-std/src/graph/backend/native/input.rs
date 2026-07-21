//! Native window input conversion into shared UI events.
//!
//! **Documentation:** `docs/pascal/std/graph/session.md`

use crate::{ConsoleKeyEvent, key_event::key_kind_index, mouse_button_index};
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta};
use winit::keyboard::{Key, ModifiersState, NamedKey};

pub(super) fn map_winit_mouse_button(button: MouseButton) -> Option<usize> {
    match button {
        MouseButton::Left => Some(mouse_button_index("Left")),
        MouseButton::Right => Some(mouse_button_index("Right")),
        MouseButton::Middle => Some(mouse_button_index("Middle")),
        MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => None,
    }
}

pub(super) fn map_winit_wheel_delta(delta: MouseScrollDelta) -> (i64, i64) {
    match delta {
        MouseScrollDelta::LineDelta(x, y) => (x.round() as i64, y.round() as i64),
        MouseScrollDelta::PixelDelta(position) => {
            (position.x.round() as i64, position.y.round() as i64)
        }
    }
}

pub(super) fn map_winit_key(
    event: &KeyEvent,
    modifiers: ModifiersState,
) -> Option<ConsoleKeyEvent> {
    if event.state != ElementState::Pressed || event.repeat {
        return None;
    }

    let shift = modifiers.shift_key();
    let ctrl = modifiers.control_key();
    let alt = modifiers.alt_key();
    let meta = modifiers.super_key();

    match &event.logical_key {
        Key::Named(NamedKey::Escape) => Some(ConsoleKeyEvent::new(
            key_kind_index("Escape"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::Tab) => Some(ConsoleKeyEvent::new(
            key_kind_index("Tab"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::Enter) => Some(ConsoleKeyEvent::new(
            key_kind_index("Enter"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::Backspace) => Some(ConsoleKeyEvent::new(
            key_kind_index("Backspace"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::Space) => Some(ConsoleKeyEvent::new(
            key_kind_index("Space"),
            ' ',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::ArrowUp) => Some(ConsoleKeyEvent::new(
            key_kind_index("Up"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::ArrowDown) => Some(ConsoleKeyEvent::new(
            key_kind_index("Down"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::ArrowLeft) => Some(ConsoleKeyEvent::new(
            key_kind_index("Left"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::ArrowRight) => Some(ConsoleKeyEvent::new(
            key_kind_index("Right"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::Home) => Some(ConsoleKeyEvent::new(
            key_kind_index("Home"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::End) => Some(ConsoleKeyEvent::new(
            key_kind_index("EndKey"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::PageUp) => Some(ConsoleKeyEvent::new(
            key_kind_index("PageUp"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::PageDown) => Some(ConsoleKeyEvent::new(
            key_kind_index("PageDown"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::Insert) => Some(ConsoleKeyEvent::new(
            key_kind_index("Insert"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::Delete) => Some(ConsoleKeyEvent::new(
            key_kind_index("Delete"),
            '\0',
            shift,
            ctrl,
            alt,
            meta,
        )),
        Key::Named(NamedKey::F1) => Some(function_key_event(1, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F2) => Some(function_key_event(2, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F3) => Some(function_key_event(3, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F4) => Some(function_key_event(4, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F5) => Some(function_key_event(5, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F6) => Some(function_key_event(6, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F7) => Some(function_key_event(7, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F8) => Some(function_key_event(8, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F9) => Some(function_key_event(9, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F10) => Some(function_key_event(10, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F11) => Some(function_key_event(11, shift, ctrl, alt, meta)),
        Key::Named(NamedKey::F12) => Some(function_key_event(12, shift, ctrl, alt, meta)),
        Key::Character(text) => map_character_key(text.as_str(), shift, ctrl, alt, meta),
        _ => event
            .text
            .as_ref()
            .and_then(|text| map_character_key(text.as_str(), shift, ctrl, alt, meta))
            .or_else(|| {
                Some(ConsoleKeyEvent::new(
                    key_kind_index("Unknown"),
                    '\0',
                    shift,
                    ctrl,
                    alt,
                    meta,
                ))
            }),
    }
}

fn function_key_event(
    number: u8,
    shift: bool,
    ctrl: bool,
    alt: bool,
    meta: bool,
) -> ConsoleKeyEvent {
    ConsoleKeyEvent::new(
        key_kind_index(&format!("F{number}")),
        '\0',
        shift,
        ctrl,
        alt,
        meta,
    )
}

fn map_character_key(
    text: &str,
    shift: bool,
    ctrl: bool,
    alt: bool,
    meta: bool,
) -> Option<ConsoleKeyEvent> {
    let ch = text.chars().next()?;
    let kind = if ch == ' ' {
        key_kind_index("Space")
    } else {
        key_kind_index("Character")
    };
    Some(ConsoleKeyEvent::new(kind, ch, shift, ctrl, alt, meta))
}

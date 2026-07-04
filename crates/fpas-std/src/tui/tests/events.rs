//! `TuiSession` console event mapping tests.

use crate::ConsoleKeyEvent;
use crate::console::{Console, KeyInput};
use crate::console_event::ConsoleEvent;
use crate::key_event::key_kind_index;

use super::helpers::test_location;
use super::{TuiEvent, TuiSession};

#[test]
fn tui_session_size_requires_open_session() {
    let session = TuiSession::default();
    let mut console = Console::new();

    let error = session
        .size(&mut console, test_location())
        .expect_err("size without open session should fail");

    assert!(
        error
            .message
            .contains("requires an open Std.Tui application session"),
        "unexpected error message: {}",
        error.message
    );
}

#[test]
fn tui_session_read_event_maps_resize_and_updates_console_size() {
    let mut session = TuiSession::default();
    let mut console = Console::new();
    let mut key_input = KeyInput::new();

    session
        .open(&mut console, &mut key_input, test_location())
        .expect("open should succeed");

    key_input.push_console_event(ConsoleEvent::resize(120, 40));

    let event = session
        .read_event(&mut console, &mut key_input, test_location())
        .expect("read event should succeed");

    assert_eq!(
        event,
        TuiEvent::Resize {
            old_width: 80,
            old_height: 25,
            width: 120,
            height: 40
        }
    );
    assert_eq!(console.screen_width(), 120);
    assert_eq!(console.screen_height(), 40);
}

#[test]
fn tui_session_ignores_invalid_resize_events() {
    let mut session = TuiSession::default();
    let mut console = Console::new();
    let mut key_input = KeyInput::new();

    session
        .open(&mut console, &mut key_input, test_location())
        .expect("open should succeed");

    key_input.push_console_event(ConsoleEvent::resize(0, 40));
    key_input.push_console_event(ConsoleEvent::resize(-1, 40));
    key_input.push_console_event(ConsoleEvent::resize(i64::from(u16::MAX) + 1, 40));
    key_input.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
        key_kind_index("Enter"),
        '\n',
        false,
        false,
        false,
        false,
    )));

    let event = session
        .read_event(&mut console, &mut key_input, test_location())
        .expect("read event should skip invalid resize events");

    assert!(matches!(event, TuiEvent::Key(_)));
    assert_eq!(console.screen_width(), 80);
    assert_eq!(console.screen_height(), 25);
}

#[test]
fn tui_session_poll_event_skips_unsupported_events_until_key() {
    let mut session = TuiSession::default();
    let mut console = Console::new();
    let mut key_input = KeyInput::new();

    session
        .open(&mut console, &mut key_input, test_location())
        .expect("open should succeed");

    key_input.push_console_event(ConsoleEvent::focus_gained());
    key_input.push_console_event(ConsoleEvent::paste("ignored".to_string()));
    key_input.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
        key_kind_index("Space"),
        ' ',
        false,
        false,
        false,
        false,
    )));

    let event = session
        .poll_event(&mut console, &mut key_input, test_location())
        .expect("poll event should succeed")
        .expect("key event should be available");

    assert!(
        matches!(event, TuiEvent::Key(ConsoleKeyEvent { kind, .. }) if kind == key_kind_index("Space"))
    );
}

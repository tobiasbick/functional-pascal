#![allow(clippy::expect_used)]

use super::*;
use crate::console::{Console, KeyInput};
use crate::console_event::ConsoleEvent;
use crate::key_event::key_kind_index;
use crate::{ConsoleKeyEvent, DamageRegion};
use fpas_bytecode::SourceLocation;

fn test_location() -> SourceLocation {
    SourceLocation::new(1, 1)
}

#[test]
fn tui_session_open_close_reopen_succeeds_without_terminal_writer() {
    let mut session = TuiSession::default();
    let mut console = Console::new();
    let mut key_input = KeyInput::new();

    session
        .open(&mut console, &mut key_input, test_location())
        .expect("first open should succeed");
    session
        .close(&mut console, &mut key_input, test_location())
        .expect("close should succeed");
    session
        .open(&mut console, &mut key_input, test_location())
        .expect("reopen should succeed");
}

#[test]
fn tui_session_second_open_is_rejected() {
    let mut session = TuiSession::default();
    let mut console = Console::new();
    let mut key_input = KeyInput::new();

    session
        .open(&mut console, &mut key_input, test_location())
        .expect("first open should succeed");

    let error = session
        .open(&mut console, &mut key_input, test_location())
        .expect_err("second open should fail");

    assert!(
        error
            .message
            .contains("cannot open a second Std.Tui session"),
        "unexpected error message: {}",
        error.message
    );
}

#[test]
fn tui_session_is_redraw_pending_peeks_without_clearing() {
    let mut session = TuiSession::default();
    let mut console = Console::new();
    let mut key_input = KeyInput::new();

    session
        .open(&mut console, &mut key_input, test_location())
        .expect("open");
    session
        .request_redraw(test_location())
        .expect("request redraw");

    assert!(
        session
            .is_redraw_pending(test_location())
            .expect("peek redraw")
    );
    assert!(
        session
            .is_redraw_pending(test_location())
            .expect("peek again")
    );
    assert_eq!(
        session
            .peek_redraw_damage(test_location())
            .expect("peek damage"),
        Some(DamageRegion::FullFrame)
    );

    let taken = session.take_redraw_pending(test_location()).expect("take");
    assert!(taken);
    assert!(
        !session
            .is_redraw_pending(test_location())
            .expect("peek after take")
    );
}

#[test]
fn tui_session_request_redraw_is_consumed_once() {
    let mut session = TuiSession::default();
    let mut console = Console::new();
    let mut key_input = KeyInput::new();

    session
        .open(&mut console, &mut key_input, test_location())
        .expect("open should succeed");
    session
        .request_redraw(test_location())
        .expect("request redraw should succeed");

    let first = session
        .take_redraw_pending(test_location())
        .expect("first redraw check should succeed");
    let second = session
        .take_redraw_pending(test_location())
        .expect("second redraw check should succeed");

    assert!(first);
    assert!(!second);
}

#[test]
fn tui_session_take_redraw_damage_returns_full_frame_once() {
    let mut session = TuiSession::default();
    let mut console = Console::new();
    let mut key_input = KeyInput::new();

    session
        .open(&mut console, &mut key_input, test_location())
        .expect("open should succeed");
    session
        .request_redraw(test_location())
        .expect("request redraw should succeed");

    let first = session
        .take_redraw_damage(test_location())
        .expect("first damage take should succeed");
    let second = session
        .take_redraw_damage(test_location())
        .expect("second damage take should succeed");

    assert_eq!(first, Some(DamageRegion::FullFrame));
    assert_eq!(second, None);
}

#[test]
fn tui_session_request_redraw_rect_marks_rect_damage() {
    let mut session = TuiSession::default();
    let mut console = Console::new();
    let mut key_input = KeyInput::new();

    session
        .open(&mut console, &mut key_input, test_location())
        .expect("open should succeed");
    session
        .request_redraw_rect(
            crate::ViewRect {
                x: 3,
                y: 4,
                width: 5,
                height: 6,
            },
            test_location(),
        )
        .expect("rect redraw should succeed");

    assert_eq!(
        session
            .peek_redraw_damage(test_location())
            .expect("peek damage"),
        Some(DamageRegion::Rect(crate::ViewRect {
            x: 3,
            y: 4,
            width: 5,
            height: 6,
        }))
    );
}

#[test]
fn tui_session_request_redraw_rect_merges_rectangles() {
    let mut session = TuiSession::default();
    let mut console = Console::new();
    let mut key_input = KeyInput::new();

    session
        .open(&mut console, &mut key_input, test_location())
        .expect("open should succeed");
    session
        .request_redraw_rect(
            crate::ViewRect {
                x: 2,
                y: 2,
                width: 4,
                height: 3,
            },
            test_location(),
        )
        .expect("first rect redraw should succeed");
    session
        .request_redraw_rect(
            crate::ViewRect {
                x: 8,
                y: 1,
                width: 2,
                height: 5,
            },
            test_location(),
        )
        .expect("second rect redraw should succeed");

    assert_eq!(
        session
            .take_redraw_damage(test_location())
            .expect("take damage"),
        Some(DamageRegion::Rect(crate::ViewRect {
            x: 2,
            y: 1,
            width: 8,
            height: 5,
        }))
    );
}

#[test]
fn tui_session_request_resize_redraw_marks_union_of_old_and_new_bounds() {
    let mut session = TuiSession::default();
    let mut console = Console::new();
    let mut key_input = KeyInput::new();

    session
        .open(&mut console, &mut key_input, test_location())
        .expect("open should succeed");
    session
        .request_resize_redraw(80, 25, 40, 10, test_location())
        .expect("resize redraw should succeed");

    assert_eq!(
        session
            .take_redraw_damage(test_location())
            .expect("take damage"),
        Some(DamageRegion::Rect(crate::ViewRect {
            x: 0,
            y: 0,
            width: 80,
            height: 25,
        }))
    );
}

#[test]
fn tui_session_request_redraw_if_absent_marks_full_frame_when_idle() {
    let mut session = TuiSession::default();
    let mut console = Console::new();
    let mut key_input = KeyInput::new();

    session
        .open(&mut console, &mut key_input, test_location())
        .expect("open should succeed");
    session
        .request_redraw_if_absent(test_location())
        .expect("conditional redraw should succeed");

    assert_eq!(
        session
            .peek_redraw_damage(test_location())
            .expect("peek damage"),
        Some(DamageRegion::FullFrame)
    );
}

#[test]
fn tui_session_request_redraw_if_absent_preserves_existing_rect_damage() {
    let mut session = TuiSession::default();
    let mut console = Console::new();
    let mut key_input = KeyInput::new();

    session
        .open(&mut console, &mut key_input, test_location())
        .expect("open should succeed");
    session
        .request_redraw_rect(
            crate::ViewRect {
                x: 6,
                y: 7,
                width: 8,
                height: 9,
            },
            test_location(),
        )
        .expect("rect redraw should succeed");
    session
        .request_redraw_if_absent(test_location())
        .expect("conditional redraw should succeed");

    assert_eq!(
        session
            .peek_redraw_damage(test_location())
            .expect("peek damage"),
        Some(DamageRegion::Rect(crate::ViewRect {
            x: 6,
            y: 7,
            width: 8,
            height: 9,
        }))
    );
}

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

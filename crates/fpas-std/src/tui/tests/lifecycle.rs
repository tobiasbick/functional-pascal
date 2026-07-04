//! `TuiSession` open, close, and headless lifecycle tests.

use crate::console::{Console, KeyInput};

use super::TuiSession;
use super::helpers::{console_with_shared_writer, test_location};

#[test]
fn tui_session_open_deferred_does_not_acquire_terminal_writer() {
    let mut session = TuiSession::default();
    let (mut console, bytes) = console_with_shared_writer();
    let mut key_input = KeyInput::new();

    session
        .open_deferred(&mut console, test_location())
        .expect("deferred open should succeed");
    session
        .close(&mut console, &mut key_input, test_location())
        .expect("close should succeed");

    assert!(
        bytes.lock().unwrap().is_empty(),
        "deferred TUI open should not emit terminal control sequences"
    );
}

#[test]
fn tui_session_open_for_test_is_headless_and_reopen_succeeds() {
    let mut session = TuiSession::default();
    let mut console = Console::new();
    let mut key_input = KeyInput::new();

    session
        .open_for_test(&mut console, test_location())
        .expect("first open_for_test should succeed");
    assert!(session.is_headless());
    session
        .close(&mut console, &mut key_input, test_location())
        .expect("close should succeed");
    assert!(!session.is_headless());

    session
        .open_for_test(&mut console, test_location())
        .expect("reopen should succeed");
    assert!(session.is_headless());
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

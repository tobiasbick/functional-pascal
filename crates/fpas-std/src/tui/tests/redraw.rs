//! `TuiSession` redraw and damage-region tests.

use crate::DamageRegion;
use crate::console::{Console, KeyInput};

use super::TuiSession;
use super::helpers::test_location;

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

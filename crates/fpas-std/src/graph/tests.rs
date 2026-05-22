#![allow(clippy::expect_used)]

use super::event::{GraphEvent, GraphEventKind};
use super::session::GraphSession;
use super::with_headless_graph_backend_for_tests;
use crate::{ConsoleKeyEvent, key_event::key_kind_index};
use fpas_bytecode::SourceLocation;

fn test_location() -> SourceLocation {
    SourceLocation::new(1, 1)
}

fn with_headless<T>(f: impl FnOnce() -> T) -> T {
    with_headless_graph_backend_for_tests(f)
}

#[test]
fn graph_session_open_and_close_work_with_headless_backend() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(320, 200, "Graph smoke", test_location())
            .expect("open should succeed");
        assert_eq!(
            session.size(test_location()).expect("size should succeed"),
            (320, 200)
        );
        session
            .close(test_location())
            .expect("close should succeed");
    });
}

#[test]
fn graph_session_open_validates_positive_surface_size() {
    with_headless(|| {
        let mut session = GraphSession::default();

        let error = session
            .open(0, 480, "Graph smoke", test_location())
            .expect_err("zero width should fail");

        assert!(
            error.message.contains("requires positive dimensions"),
            "message={}",
            error.message
        );
    });
}

#[test]
fn graph_session_upload_frame_rejects_pixel_length_mismatch() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(2, 2, "Graph smoke", test_location())
            .expect("open should succeed");

        let error = session
            .upload_frame(2, 2, &[1, 2, 3], test_location())
            .expect_err("pixel length mismatch should fail");

        assert!(
            error.message.contains("expected 4 pixels for 2x2, got 3"),
            "message={}",
            error.message
        );
    });
}

#[test]
fn graph_session_upload_frame_rejects_surface_size_mismatch() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(2, 2, "Graph smoke", test_location())
            .expect("open should succeed");

        let error = session
            .upload_frame(3, 2, &[1, 2, 3, 4, 5, 6], test_location())
            .expect_err("surface mismatch should fail");

        assert!(
            error
                .message
                .contains("expected Width=2 and Height=2, got Width=3 and Height=2"),
            "message={}",
            error.message
        );
    });
}

#[test]
fn graph_session_upload_frame_accepts_valid_rgb24_payload() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(2, 1, "Graph smoke", test_location())
            .expect("open should succeed");
        session
            .upload_frame(2, 1, &[0x00102040, 0x00FF00AA], test_location())
            .expect("valid frame should succeed");

        let frame = session
            .last_uploaded_frame_for_tests()
            .expect("frame should be staged");
        assert_eq!(frame.width, 2);
        assert_eq!(frame.height, 1);
        assert_eq!(frame.pixels, vec![0x00102040, 0x00FF00AA]);
    });
}

#[test]
fn graph_session_size_and_close_follow_open_state() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(320, 200, "Graph smoke", test_location())
            .expect("open should succeed");

        assert_eq!(
            session.size(test_location()).expect("size should succeed"),
            (320, 200)
        );

        session
            .close(test_location())
            .expect("close should succeed");
        let error = session
            .size(test_location())
            .expect_err("size after close should fail");
        assert!(
            error.message.contains("requires an open graphics session"),
            "message={}",
            error.message
        );
    });
}

#[test]
fn graph_session_poll_event_returns_queued_event() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(320, 200, "Graph smoke", test_location())
            .expect("open should succeed");
        session.push_event_for_tests(GraphEvent::Resize {
            width: 800,
            height: 600,
        });

        let event = session
            .poll_event(test_location())
            .expect("poll should succeed");
        assert_eq!(
            event,
            Some(GraphEvent::Resize {
                width: 800,
                height: 600,
            })
        );
    });
}

#[test]
fn graph_event_kind_matches_payload_variant() {
    assert_eq!(
        GraphEvent::CloseRequested.kind(),
        GraphEventKind::CloseRequested
    );
    assert_eq!(
        GraphEvent::Resize {
            width: 640,
            height: 480,
        }
        .kind(),
        GraphEventKind::Resize
    );
    assert_eq!(
        GraphEvent::Key(ConsoleKeyEvent::new(
            key_kind_index("Escape"),
            '\0',
            false,
            false,
            false,
            false,
        ))
        .kind(),
        GraphEventKind::Key
    );
}

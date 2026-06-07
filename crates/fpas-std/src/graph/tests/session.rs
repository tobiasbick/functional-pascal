//! Graph session lifecycle tests.

use super::super::session::GraphSession;
use super::common::{test_location, with_headless};
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
fn graph_session_close_is_a_noop_after_the_session_is_already_closed() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(320, 200, "Graph smoke", test_location())
            .expect("open should succeed");
        session
            .close(test_location())
            .expect("first close should succeed");
        session
            .close(test_location())
            .expect("second close should be a no-op");
    });
}

#[test]
fn graph_session_can_open_close_and_reopen_without_stale_backend_state() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(320, 200, "Graph smoke", test_location())
            .expect("first open should succeed");
        session
            .close(test_location())
            .expect("first close should succeed");

        session
            .open(160, 120, "Graph smoke 2", test_location())
            .expect("second open should succeed");
        assert_eq!(session.backbuffer_size_for_tests(), (160, 120));
        assert_eq!(
            session.size(test_location()).expect("size should succeed"),
            (160, 120)
        );
        session
            .close(test_location())
            .expect("second close should succeed");
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
fn graph_session_open_rejects_zero_height() {
    with_headless(|| {
        let mut session = GraphSession::default();

        let error = session
            .open(320, 0, "Graph smoke", test_location())
            .expect_err("zero height should fail");

        assert!(
            error.message.contains("requires positive dimensions"),
            "message={}",
            error.message
        );
    });
}

#[test]
fn graph_session_second_open_is_rejected() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(320, 200, "Graph smoke", test_location())
            .expect("first open should succeed");

        let error = session
            .open(160, 120, "Graph two", test_location())
            .expect_err("second open should fail");

        assert!(
            error
                .message
                .contains("cannot open a second graphics session"),
            "message={}",
            error.message
        );
    });
}

#[test]
fn graph_session_mutations_require_open_session() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(2, 2, "Graph smoke", test_location())
            .expect("open should succeed");
        session
            .close(test_location())
            .expect("close should succeed");

        let error = session
            .clear(0, test_location())
            .expect_err("clear after close should fail");

        assert!(
            error.message.contains("requires an open graphics session"),
            "message={}",
            error.message
        );
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

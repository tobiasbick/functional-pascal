//! Graph session validation and error-path tests.

use super::super::session::GraphSession;
use super::common::{test_location, with_headless};
#[test]
fn graph_session_clear_rejects_invalid_rgb24_color() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(2, 2, "Graph smoke", test_location())
            .expect("open should succeed");

        let error = session
            .clear(0x0100_0000, test_location())
            .expect_err("out-of-range color should fail");

        assert!(
            error.message.contains("requires `$00RRGGBB` colors"),
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
fn graph_session_upload_frame_rejects_out_of_range_pixel() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(2, 1, "Graph smoke", test_location())
            .expect("open should succeed");

        let error = session
            .upload_frame(2, 1, &[0x0010_2040, 0x0100_0000], test_location())
            .expect_err("out-of-range pixel should fail");

        assert!(
            error.message.contains("is out of range"),
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
fn graph_session_open_rejects_oversized_surface() {
    with_headless(|| {
        let mut session = GraphSession::default();
        let error = session
            .open(100_000, 100_000, "too big", test_location())
            .expect_err("oversized surface should fail");

        assert!(
            error.message.contains("exceeds the maximum"),
            "message={}",
            error.message
        );
    });
}

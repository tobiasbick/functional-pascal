//! Graph backbuffer staging and presentation tests.

use super::super::{session::GraphSession, set_headless_graph_surface_size_for_tests};
use super::common::{test_location, with_headless};

#[test]
fn graph_session_clear_fills_runtime_backbuffer() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(3, 2, "Graph smoke", test_location())
            .expect("open should succeed");

        session
            .clear(0x00102040, test_location())
            .expect("clear should succeed");

        assert_eq!(
            session.backbuffer_pixels_for_tests(),
            &[
                0x00102040, 0x00102040, 0x00102040, 0x00102040, 0x00102040, 0x00102040
            ]
        );
    });
}

#[test]
fn graph_session_put_pixel_writes_inside_and_clips_outside() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(2, 2, "Graph smoke", test_location())
            .expect("open should succeed");
        session
            .clear(0, test_location())
            .expect("clear should succeed");

        session
            .put_pixel(1, 0, 0x00ABCDEF, test_location())
            .expect("put pixel should succeed");
        session
            .put_pixel(-1, 0, 0x00FFFFFF, test_location())
            .expect("clipped write should still succeed");
        session
            .put_pixel(99, 99, 0x00FFFFFF, test_location())
            .expect("clipped write should still succeed");

        assert_eq!(
            session.backbuffer_pixels_for_tests(),
            &[0x00000000, 0x00ABCDEF, 0x00000000, 0x00000000]
        );
    });
}

#[test]
fn graph_session_upload_frame_stages_valid_payload_in_backbuffer_and_cache() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(2, 1, "Graph smoke", test_location())
            .expect("open should succeed");
        session
            .upload_frame(2, 1, &[0x00102040, 0x00FF00AA], test_location())
            .expect("valid frame should succeed");

        assert_eq!(
            session.backbuffer_pixels_for_tests(),
            &[0x00102040, 0x00FF00AA]
        );

        let frame = session
            .last_uploaded_frame_for_tests()
            .expect("frame should be staged");
        assert_eq!(frame.width, 2);
        assert_eq!(frame.height, 1);
        assert_eq!(frame.pixels, vec![0x00102040, 0x00FF00AA]);
    });
}

#[test]
fn graph_session_upload_frame_uses_last_observed_size_during_resize_race() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(2, 1, "Graph smoke", test_location())
            .expect("open should succeed");
        assert_eq!(
            session.size(test_location()).expect("size should succeed"),
            (2, 1)
        );

        assert!(set_headless_graph_surface_size_for_tests(4, 3).is_ok());

        session
            .upload_frame(2, 1, &[0x00102040, 0x00FF00AA], test_location())
            .expect("stale frame size should not fail during a concurrent resize");

        let frame = session
            .last_uploaded_frame_for_tests()
            .expect("frame should still be staged");
        assert_eq!(frame.width, 2);
        assert_eq!(frame.height, 1);
        assert_eq!(frame.pixels, vec![0x00102040, 0x00FF00AA]);
    });
}

#[test]
fn graph_session_present_accepts_runtime_owned_backbuffer() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(2, 1, "Graph smoke", test_location())
            .expect("open should succeed");
        session
            .clear(0x00010203, test_location())
            .expect("clear should succeed");
        session
            .put_pixel(1, 0, 0x00ABCDEF, test_location())
            .expect("put pixel should succeed");
        session
            .present(test_location())
            .expect("present should succeed");

        assert_eq!(
            session.backbuffer_pixels_for_tests(),
            &[0x00010203, 0x00ABCDEF]
        );
    });
}

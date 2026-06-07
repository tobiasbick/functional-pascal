//! Graph drawing primitive tests.

use super::super::session::GraphSession;
use super::common::{test_location, with_headless};
#[test]
fn graph_session_draw_line_clips_to_surface_bounds() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(3, 3, "Graph smoke", test_location())
            .expect("open should succeed");
        session
            .draw_line(-1, 1, 3, 1, 0x00000001, test_location())
            .expect("draw line should succeed");

        assert_eq!(
            session.backbuffer_pixels_for_tests(),
            &[0, 0, 0, 0x00000001, 0x00000001, 0x00000001, 0, 0, 0]
        );
    });
}

#[test]
fn graph_session_draw_rect_renders_outline() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(5, 4, "Graph smoke", test_location())
            .expect("open should succeed");
        session
            .draw_rect(1, 0, 3, 3, 0x00000002, test_location())
            .expect("draw rect should succeed");

        assert_eq!(
            session.backbuffer_pixels_for_tests(),
            &[
                0, 0x00000002, 0x00000002, 0x00000002, 0, 0, 0x00000002, 0, 0x00000002, 0, 0,
                0x00000002, 0x00000002, 0x00000002, 0, 0, 0, 0, 0, 0,
            ]
        );
    });
}

#[test]
fn graph_session_fill_rect_fills_area() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(4, 3, "Graph smoke", test_location())
            .expect("open should succeed");
        session
            .fill_rect(1, 1, 2, 2, 0x00000003, test_location())
            .expect("fill rect should succeed");

        assert_eq!(
            session.backbuffer_pixels_for_tests(),
            &[
                0, 0, 0, 0, 0, 0x00000003, 0x00000003, 0, 0, 0x00000003, 0x00000003, 0
            ]
        );
    });
}

#[test]
fn graph_session_draw_circle_renders_outline() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(5, 5, "Graph smoke", test_location())
            .expect("open should succeed");
        session
            .draw_circle(2, 2, 1, 0x00000004, test_location())
            .expect("draw circle should succeed");

        assert_eq!(
            session.backbuffer_pixels_for_tests(),
            &[
                0, 0, 0, 0, 0, 0, 0, 0x00000004, 0, 0, 0, 0x00000004, 0, 0x00000004, 0, 0, 0,
                0x00000004, 0, 0, 0, 0, 0, 0, 0,
            ]
        );
    });
}

#[test]
fn graph_session_draw_circle_rejects_negative_radius() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(5, 5, "Graph smoke", test_location())
            .expect("open should succeed");

        let error = session
            .draw_circle(2, 2, -1, 0x00000004, test_location())
            .expect_err("negative radius should fail");

        assert!(
            error.message.contains("requires a non-negative radius"),
            "message={}",
            error.message
        );
    });
}

#[test]
fn graph_session_draw_circle_with_zero_radius_draws_center_pixel() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(3, 3, "Graph smoke", test_location())
            .expect("open should succeed");
        session
            .clear(0, test_location())
            .expect("clear should succeed");
        session
            .draw_circle(1, 1, 0, 0x00000007, test_location())
            .expect("zero radius should succeed");

        assert_eq!(
            session.backbuffer_pixels_for_tests(),
            &[0, 0, 0, 0, 0x00000007, 0, 0, 0, 0]
        );
    });
}

#[test]
fn graph_session_draw_rect_rejects_non_positive_dimensions() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(4, 4, "Graph smoke", test_location())
            .expect("open should succeed");

        let error = session
            .draw_rect(0, 0, 0, 2, 0x00000002, test_location())
            .expect_err("zero width should fail");

        assert!(
            error.message.contains("requires positive dimensions"),
            "message={}",
            error.message
        );
    });
}

#[test]
fn graph_session_draw_rect_1x1_fills_single_pixel() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(3, 3, "Graph smoke", test_location())
            .expect("open should succeed");
        session
            .clear(0, test_location())
            .expect("clear should succeed");
        session
            .draw_rect(1, 1, 1, 1, 0x00000008, test_location())
            .expect("1x1 rect should succeed");

        assert_eq!(
            session.backbuffer_pixels_for_tests(),
            &[0, 0, 0, 0, 0x00000008, 0, 0, 0, 0]
        );
    });
}

#[test]
fn graph_session_draw_text_renders_deterministic_bitmap_glyphs() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(5, 7, "Graph text", test_location())
            .expect("open should succeed");
        session
            .draw_text(0, 0, "A", 0x00000005, test_location())
            .expect("draw text should succeed");

        assert_eq!(
            session.backbuffer_pixels_for_tests(),
            &[
                0, 0x00000005, 0x00000005, 0x00000005, 0, 0x00000005, 0, 0, 0, 0x00000005,
                0x00000005, 0, 0, 0, 0x00000005, 0x00000005, 0x00000005, 0x00000005, 0x00000005,
                0x00000005, 0x00000005, 0, 0, 0, 0x00000005, 0x00000005, 0, 0, 0, 0x00000005,
                0x00000005, 0, 0, 0, 0x00000005,
            ]
        );
    });
}

#[test]
fn graph_session_draw_text_clips_outside_surface_bounds() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(5, 7, "Graph text", test_location())
            .expect("open should succeed");
        session
            .draw_text(3, 0, "A", 0x00000006, test_location())
            .expect("draw text should succeed");

        assert_eq!(
            session.backbuffer_pixels_for_tests(),
            &[
                0, 0, 0, 0, 0x00000006, 0, 0, 0, 0x00000006, 0, 0, 0, 0, 0x00000006, 0, 0, 0, 0,
                0x00000006, 0x00000006, 0, 0, 0, 0x00000006, 0, 0, 0, 0, 0x00000006, 0, 0, 0, 0,
                0x00000006, 0,
            ]
        );
    });
}

#[test]
fn graph_session_draw_text_unknown_char_renders_question_mark_glyph() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(5, 7, "Graph text", test_location())
            .expect("open should succeed");
        session
            .clear(0, test_location())
            .expect("clear should succeed");
        session
            .draw_text(0, 0, "@", 0x00000009, test_location())
            .expect("draw text should succeed");
        let unknown_pixels = session.backbuffer_pixels_for_tests().to_vec();

        session
            .clear(0, test_location())
            .expect("clear should succeed");
        session
            .draw_text(0, 0, "?", 0x00000009, test_location())
            .expect("draw text should succeed");

        assert_eq!(unknown_pixels, session.backbuffer_pixels_for_tests());
    });
}

#[test]
fn graph_session_draw_text_normalizes_lowercase_to_uppercase_glyphs() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(5, 7, "Graph text", test_location())
            .expect("open should succeed");
        session
            .clear(0, test_location())
            .expect("clear should succeed");
        session
            .draw_text(0, 0, "a", 0x0000000A, test_location())
            .expect("draw text should succeed");
        let lowercase_pixels = session.backbuffer_pixels_for_tests().to_vec();

        session
            .clear(0, test_location())
            .expect("clear should succeed");
        session
            .draw_text(0, 0, "A", 0x0000000A, test_location())
            .expect("draw text should succeed");

        assert_eq!(lowercase_pixels, session.backbuffer_pixels_for_tests());
    });
}

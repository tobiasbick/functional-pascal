#![allow(clippy::expect_used)]

use super::event::{GraphEvent, GraphEventKind};
use super::session::GraphSession;
use super::{set_headless_graph_surface_size_for_tests, with_headless_graph_backend_for_tests};
use crate::{ConsoleKeyEvent, key_event::key_kind_index, mouse_action_index, mouse_button_index};
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
fn graph_session_upload_frame_updates_runtime_backbuffer() {
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
    });
}

#[test]
fn graph_session_upload_frame_uses_last_observed_size_during_resize_race() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(2, 1, "Graph smoke", test_location())
            .expect("open should succeed");
        assert_eq!(session.size(test_location()).expect("size should succeed"), (2, 1));

        set_headless_graph_surface_size_for_tests(4, 3);

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
fn graph_session_resize_event_reallocates_runtime_backbuffer() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(2, 2, "Graph smoke", test_location())
            .expect("open should succeed");
        session.push_event_for_tests(GraphEvent::Resize {
            width: 3,
            height: 1,
        });

        let event = session
            .poll_event(test_location())
            .expect("poll should succeed");
        assert_eq!(
            event,
            Some(GraphEvent::Resize {
                width: 3,
                height: 1,
            })
        );
        assert_eq!(session.backbuffer_size_for_tests(), (3, 1));
        assert_eq!(session.backbuffer_pixels_for_tests(), &[0, 0, 0]);
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
fn graph_session_poll_event_returns_queued_mouse_event() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(320, 200, "Graph smoke", test_location())
            .expect("open should succeed");
        session.push_event_for_tests(GraphEvent::Mouse {
            action: mouse_action_index("Drag"),
            button: mouse_button_index("Left"),
            x: 12,
            y: 34,
            shift: true,
            ctrl: false,
            alt: false,
            meta: false,
        });

        let event = session
            .poll_event(test_location())
            .expect("poll should succeed");
        assert_eq!(
            event,
            Some(GraphEvent::Mouse {
                action: mouse_action_index("Drag"),
                button: mouse_button_index("Left"),
                x: 12,
                y: 34,
                shift: true,
                ctrl: false,
                alt: false,
                meta: false,
            })
        );
    });
}

#[test]
fn graph_session_poll_event_returns_queued_wheel_event() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(320, 200, "Graph smoke", test_location())
            .expect("open should succeed");
        session.push_event_for_tests(GraphEvent::Wheel {
            delta_x: -1,
            delta_y: 2,
            x: 20,
            y: 40,
            shift: false,
            ctrl: true,
            alt: false,
            meta: false,
        });

        let event = session
            .poll_event(test_location())
            .expect("poll should succeed");
        assert_eq!(
            event,
            Some(GraphEvent::Wheel {
                delta_x: -1,
                delta_y: 2,
                x: 20,
                y: 40,
                shift: false,
                ctrl: true,
                alt: false,
                meta: false,
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
    assert_eq!(
        GraphEvent::Mouse {
            action: mouse_action_index("Move"),
            button: mouse_button_index("None"),
            x: 0,
            y: 0,
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
        .kind(),
        GraphEventKind::Mouse
    );
    assert_eq!(
        GraphEvent::Wheel {
            delta_x: 0,
            delta_y: -1,
            x: 0,
            y: 0,
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
        .kind(),
        GraphEventKind::Wheel
    );
}

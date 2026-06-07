//! Graph host UI event tests.

use super::super::event::{GraphEvent, GraphEventKind};
use super::super::session::GraphSession;
use super::common::{test_location, with_headless};
use crate::{ConsoleKeyEvent, key_event::key_kind_index, mouse_action_index, mouse_button_index};
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
            .read_host_ui_event_timeout(0, test_location())
            .expect("read host ui event should succeed")
            .and_then(GraphEvent::from_ui_event);
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
fn graph_session_read_host_ui_event_returns_queued_event() {
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
            .read_host_ui_event_timeout(0, test_location())
            .expect("read host ui event should succeed")
            .and_then(GraphEvent::from_ui_event);
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
fn graph_session_read_host_ui_event_timeout_returns_queued_event_immediately() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(320, 200, "Graph smoke", test_location())
            .expect("open should succeed");
        session.push_event_for_tests(GraphEvent::Resize {
            width: 640,
            height: 480,
        });

        let event = session
            .read_host_ui_event_timeout(16, test_location())
            .expect("read host ui event timeout should succeed")
            .and_then(GraphEvent::from_ui_event);
        assert_eq!(
            event,
            Some(GraphEvent::Resize {
                width: 640,
                height: 480,
            })
        );
    });
}

#[test]
fn graph_session_read_host_ui_event_returns_queued_close_requested_event() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(320, 200, "Graph smoke", test_location())
            .expect("open should succeed");
        session.push_event_for_tests(GraphEvent::CloseRequested);

        let event = session
            .read_host_ui_event_timeout(0, test_location())
            .expect("read host ui event should succeed")
            .and_then(GraphEvent::from_ui_event);
        assert_eq!(event, Some(GraphEvent::CloseRequested));
    });
}

#[test]
fn graph_session_read_host_ui_event_returns_queued_key_event() {
    with_headless(|| {
        let mut session = GraphSession::default();
        session
            .open(320, 200, "Graph smoke", test_location())
            .expect("open should succeed");
        session.push_event_for_tests(GraphEvent::Key(ConsoleKeyEvent::new(
            key_kind_index("Escape"),
            '\0',
            false,
            false,
            false,
            false,
        )));

        let event = session
            .read_host_ui_event_timeout(0, test_location())
            .expect("read host ui event should succeed")
            .and_then(GraphEvent::from_ui_event);
        assert_eq!(
            event,
            Some(GraphEvent::Key(ConsoleKeyEvent::new(
                key_kind_index("Escape"),
                '\0',
                false,
                false,
                false,
                false,
            )))
        );
    });
}

#[test]
fn graph_session_read_host_ui_event_returns_queued_mouse_event() {
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
            .read_host_ui_event_timeout(0, test_location())
            .expect("read host ui event should succeed")
            .and_then(GraphEvent::from_ui_event);
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
fn graph_session_read_host_ui_event_returns_queued_wheel_event() {
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
            .read_host_ui_event_timeout(0, test_location())
            .expect("read host ui event should succeed")
            .and_then(GraphEvent::from_ui_event);
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

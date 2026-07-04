use fpas_std::{
    ConsoleKeyEvent, GraphEvent, key_event::key_kind_index, last_headless_graph_frame_for_tests,
};

use super::support::{
    JULIA_GRAPH_EXAMPLE, MANDELBROT_GRAPH_EXAMPLE, compile_run_with_graph_events, with_headless,
};

#[test]
fn std_graph_julia_example_renders_one_headless_frame_then_exits_on_escape() {
    with_headless(|| {
        let out = compile_run_with_graph_events(
            JULIA_GRAPH_EXAMPLE,
            &[GraphEvent::Key(ConsoleKeyEvent::new(
                key_kind_index("Escape"),
                '\0',
                false,
                false,
                false,
                false,
            ))],
        );

        assert!(out.lines.is_empty(), "unexpected output: {:?}", out.lines);

        let frame = last_headless_graph_frame_for_tests()
            .expect("present should publish a headless frame snapshot");
        assert_eq!(frame.width(), 96);
        assert_eq!(frame.height(), 72);
        assert!(
            frame.pixels().iter().any(|pixel| *pixel != 0),
            "expected a non-empty Julia frame"
        );
    });
}

#[test]
fn std_graph_mandelbrot_example_renders_one_headless_frame_then_exits_on_escape() {
    with_headless(|| {
        let out = compile_run_with_graph_events(
            MANDELBROT_GRAPH_EXAMPLE,
            &[GraphEvent::Key(ConsoleKeyEvent::new(
                key_kind_index("Escape"),
                '\0',
                false,
                false,
                false,
                false,
            ))],
        );

        assert!(out.lines.is_empty(), "unexpected output: {:?}", out.lines);

        let frame = last_headless_graph_frame_for_tests()
            .expect("present should publish a headless frame snapshot");
        assert_eq!(frame.width(), 1280);
        assert_eq!(frame.height(), 800);
        assert!(
            frame.pixels().iter().any(|pixel| *pixel != 0),
            "expected a non-empty Mandelbrot frame"
        );
    });
}

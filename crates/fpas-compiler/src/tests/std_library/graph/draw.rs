use fpas_std::{
    ConsoleKeyEvent, GraphEvent, key_event::key_kind_index, last_headless_graph_frame_for_tests,
};

use super::super::super::compile_and_run;
use super::support::{GRAPH_BASICS_EXAMPLE, compile_run_with_graph_events, with_headless};

#[test]
fn std_graph_clear_put_pixel_and_present_render_headless_frame() {
    with_headless(|| {
        let out = compile_and_run(
            "\
program T;
uses Std.Graph;

begin
  var App: Application := Application.Open(2, 1, 'Graph draw');
  Application.Clear(App, $00010203);
  Application.PutPixel(App, 1, 0, $00ABCDEF);
  Application.Present(App)
end.",
        );

        assert!(out.lines.is_empty(), "unexpected output: {:?}", out.lines);

        let frame = last_headless_graph_frame_for_tests()
            .expect("present should publish a headless frame snapshot");
        assert_eq!(frame.width(), 2);
        assert_eq!(frame.height(), 1);
        assert_eq!(frame.pixels(), &[0x00010203, 0x00ABCDEF]);
    });
}

#[test]
fn std_graph_draw_primitives_render_headless_frame() {
    with_headless(|| {
        let out = compile_and_run(
            "\
program T;
uses Std.Graph;

begin
  var App: Application := Application.Open(5, 5, 'Graph draw');
  Application.DrawLine(App, -1, 1, 3, 1, $00000001);
  Application.DrawRect(App, 1, 0, 3, 3, $00000002);
  Application.FillRect(App, 0, 4, 2, 1, $00000003);
  Application.DrawCircle(App, 3, 3, 1, $00000004);
  Application.Present(App)
end.",
        );

        assert!(out.lines.is_empty(), "unexpected output: {:?}", out.lines);

        let frame = last_headless_graph_frame_for_tests()
            .expect("present should publish a headless frame snapshot");
        assert_eq!(frame.width(), 5);
        assert_eq!(frame.height(), 5);
        assert_eq!(
            frame.pixels(),
            &[
                0, 0x00000002, 0x00000002, 0x00000002, 0, 0x00000001, 0x00000002, 0x00000001,
                0x00000002, 0, 0, 0x00000002, 0x00000002, 0x00000004, 0, 0, 0, 0x00000004, 0,
                0x00000004, 0x00000003, 0x00000003, 0, 0x00000004, 0,
            ]
        );
    });
}

#[test]
fn std_graph_draw_text_renders_headless_frame() {
    with_headless(|| {
        let out = compile_and_run(
            "\
program T;
uses Std.Graph;

begin
  var App: Application := Application.Open(5, 7, 'Graph text');
  Application.DrawText(App, 0, 0, 'A', $00000005);
  Application.Present(App)
end.",
        );

        assert!(out.lines.is_empty(), "unexpected output: {:?}", out.lines);

        let frame = last_headless_graph_frame_for_tests()
            .expect("present should publish a headless frame snapshot");
        assert_eq!(frame.width(), 5);
        assert_eq!(frame.height(), 7);
        assert_eq!(
            frame.pixels(),
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
fn std_graph_basics_example_runs_headless() {
    with_headless(|| {
        let out = compile_run_with_graph_events(
            GRAPH_BASICS_EXAMPLE,
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
        assert_eq!(frame.width(), 32);
        assert_eq!(frame.height(), 24);
    });
}

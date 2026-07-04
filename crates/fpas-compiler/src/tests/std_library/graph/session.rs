use super::super::super::{compile_and_run, compile_err, compile_run_error};
use super::support::with_headless;

#[test]
fn std_graph_open_rejects_second_session() {
    with_headless(|| {
        let error = compile_run_error(
            "\
program T;
uses Std.Graph;

begin
  var First: Application := Application.Open(2, 2, 'Graph one');
  var Second: Application := Application.Open(3, 1, 'Graph two')
end.",
        );

        assert!(
            error
                .message
                .contains("cannot open a second graphics session"),
            "unexpected runtime error: {}",
            error.message
        );
    });
}

#[test]
fn std_graph_clear_rejects_invalid_rgb24_color() {
    with_headless(|| {
        let error = compile_run_error(
            "\
program T;
uses Std.Graph;

begin
  var App: Application := Application.Open(2, 2, 'Graph smoke');
  Application.Clear(App, $01000000)
end.",
        );

        assert!(
            error.message.contains("requires `$00RRGGBB` colors"),
            "unexpected runtime error: {}",
            error.message
        );
    });
}

#[test]
fn std_graph_open_rejects_wrong_argument_count_during_compilation() {
    let err = compile_err(
        "\
program T;
uses Std.Graph;

begin
  var App: Application := Application.Open(640, 480)
end.",
    );

    assert!(
        err.message.contains("expects 3 arguments"),
        "unexpected compiler error: {}",
        err.message
    );
}

#[test]
fn std_graph_phase1_runtime_bridge_supports_size_upload_and_close() {
    with_headless(|| {
        let out = compile_and_run(
            "\
program T;
uses Std.Console, Std.Graph, Std.Option;

begin
    var App: Application := Application.Open(2, 2, 'Graph smoke');
  var Screen: Size := Application.Size(App);
  Std.Console.WriteLn(Screen.width);
  Std.Console.WriteLn(Screen.height);
  var Pixels: array of integer := [$00102040, $00010203, $00000000, $00FFFFFF];
  Application.UploadFrame(App, 2, 2, Pixels);
  Application.Close(App)
end.",
        );

        assert_eq!(out.lines, vec!["2", "2"]);
    });
}

#[test]
fn std_graph_can_close_and_reopen_in_one_process_without_stale_state() {
    with_headless(|| {
        let out = compile_and_run(
            "\
program T;
uses Std.Console, Std.Conv, Std.Graph;

begin
  var First: Application := Application.Open(2, 2, 'Graph one');
  var FirstSize: Size := Application.Size(First);
  Std.Console.WriteLn(IntToStr(FirstSize.width), 'x', IntToStr(FirstSize.height));
  Application.Close(First);

  var Second: Application := Application.Open(3, 1, 'Graph two');
  var SecondSize: Size := Application.Size(Second);
  Std.Console.WriteLn(IntToStr(SecondSize.width), 'x', IntToStr(SecondSize.height));
  Application.Close(Second)
end.",
        );

        assert_eq!(out.lines, vec!["2x2", "3x1"]);
    });
}

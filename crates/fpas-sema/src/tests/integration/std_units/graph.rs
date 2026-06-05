//! Semantic integration tests for `Std.Graph`.
//!
//! **Documentation:** `docs/pascal/std/graph.md` (from the repository root).

use super::{check_errors, check_ok};

#[test]
fn std_graph_application_surface_is_available() {
    check_ok(
        "\
program T;
uses Std.Graph;
begin
  var App: Application := Application.Open(640, 480, 'Graph smoke');
  var Screen: Size := Application.Size(App);
  Application.RequestRedraw(App);
  var Pixels: array of integer := [$00102040, $00102040];
  var Kind: EventKind := EventKind.Resize;
    var MouseKind: EventKind := EventKind.Mouse;
    var WheelKind: EventKind := EventKind.Wheel;
  var Width: integer := Screen.width;
  var IsEscape: boolean := Std.Console.KeyKind.Escape = Std.Console.KeyKind.Escape;
    var ActionValue: Std.Console.MouseAction := Std.Console.MouseAction.Down;
    var ButtonValue: Std.Console.MouseButton := Std.Console.MouseButton.Left;
    Application.Clear(App, $00010203);
    Application.PutPixel(App, 1, 0, $00ABCDEF);
    Application.DrawLine(App, 0, 0, 3, 3, $00000010);
    Application.DrawRect(App, 1, 1, 2, 2, $00000020);
    Application.FillRect(App, 0, 0, 1, 1, $00000030);
    Application.DrawCircle(App, 3, 3, 1, $00000040);
    Application.DrawText(App, 0, 0, 'A', $00000050);
    Application.Present(App);
  Application.UploadFrame(App, 1, 2, Pixels);
  Application.Close(App)
end.",
    );
}

#[test]
fn std_graph_mouse_and_wheel_event_fields_are_available() {
    check_ok(
        "\
program T;
uses Std.Graph;
begin
  var App: Application := Application.Open(320, 200, 'Graph events');
  var Kind: EventKind := EventKind.Mouse;
    var WheelKind: EventKind := EventKind.Wheel;
    var Action: Std.Console.MouseAction := Std.Console.MouseAction.Down;
    var Button: Std.Console.MouseButton := Std.Console.MouseButton.Left;
    var X: integer := 12;
    var Y: integer := 34;
    var WheelX: integer := -1;
    var WheelY: integer := 2;
    var Shifted: boolean := true
end.",
    );
}

#[test]
fn std_graph_fully_qualified_call_works_without_uses_clause() {
    check_ok(
        "\
program T;
begin
  Std.Graph.Application.Close(Std.Graph.Application.Open(320, 200, 'G'))
end.",
    );
}

#[test]
fn std_graph_short_name_requires_uses() {
    let errs = check_errors(
        "\
program T;
begin
  var App: Application := Application.Open(640, 480, 'Graph smoke')
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Unknown type")),
        "{errs:#?}"
    );
}

#[test]
fn std_graph_application_open_wrong_arg_count() {
    let errs = check_errors(
        "\
program T;
uses Std.Graph;
begin
  Application.Open(640, 480)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("expects 3 arguments, got 2")),
        "{errs:#?}"
    );
}

#[test]
fn std_graph_upload_frame_requires_integer_array() {
    let errs = check_errors(
        "\
program T;
uses Std.Graph;
begin
  var App: Application := Application.Open(640, 480, 'Graph smoke');
  Application.UploadFrame(App, 1, 1, ['x'])
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("array of integer") || e.message.contains("expected")),
        "{errs:#?}"
    );
}

#[test]
fn std_graph_present_wrong_arg_count() {
    let errs = check_errors(
        "\
program T;
uses Std.Graph;
begin
  var App: Application := Application.Open(640, 480, 'Graph smoke');
  Application.Present(App, App)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("expects 1 arguments, got 2")),
        "{errs:#?}"
    );
}

#[test]
fn std_graph_draw_circle_wrong_arg_count() {
    let errs = check_errors(
        "\
program T;
uses Std.Graph;
begin
  var App: Application := Application.Open(640, 480, 'Graph smoke');
  Application.DrawCircle(App, 10, 10, 3)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("expects 5 arguments, got 4")),
        "{errs:#?}"
    );
}

#[test]
fn std_graph_draw_text_wrong_arg_count() {
    let errs = check_errors(
        "\
program T;
uses Std.Graph;
begin
  var App: Application := Application.Open(640, 480, 'Graph smoke');
  Application.DrawText(App, 10, 10, 'A')
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("expects 5 arguments, got 4")),
        "{errs:#?}"
    );
}

#[test]
fn std_graph_size_unknown_field() {
    let errs = check_errors(
        "\
program T;
uses Std.Graph;
begin
  var App: Application := Application.Open(640, 480, 'Graph smoke');
  var Screen: Size := Application.Size(App);
  var Depth: integer := Screen.depth
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("no field")),
        "{errs:#?}"
    );
}

#[test]
fn std_graph_event_kind_unknown_member() {
    let errs = check_errors(
        "\
program T;
uses Std.Graph;
begin
    var Kind: EventKind := Std.Graph.EventKind.Touch
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("Undefined") || e.message.contains("unknown")),
        "{errs:#?}"
    );
}

#[test]
fn uses_std_graph_case_insensitive() {
    check_ok(
        "\
program T;
uses std.graph;
begin
  var App: Application := Application.Open(320, 200, 'Graph smoke');
  Application.Close(App)
end.",
    );
}

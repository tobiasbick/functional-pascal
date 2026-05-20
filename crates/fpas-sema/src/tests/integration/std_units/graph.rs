//! Semantic integration tests for `Std.Graph`.
//!
//! **Documentation:** `docs/future/std.graph/01-mvp.md`, `docs/future/std.graph/02-pascal-surface.md` (from the repository root).

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
  var Pending: Option of Event := Application.PollEvent(App);
  var Pixels: array of integer := [$00102040, $00102040];
  var Kind: EventKind := EventKind.Resize;
  var Width: integer := Screen.width;
  var IsEscape: boolean := Std.Console.KeyKind.Escape = Std.Console.KeyKind.Escape;
  Application.UploadFrame(App, 1, 2, Pixels);
  Application.Close(App)
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
  var Kind: EventKind := Std.Graph.EventKind.Mouse
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
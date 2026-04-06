use super::{check_errors, check_ok};

#[test]
fn std_tui_application_surface_is_available() {
    check_ok(
        "\
program T;
uses Std.Tui;
begin
  var App: Application := Application.Open();
  var Screen: Size := Application.Size(App);
  var Ev: Event := Application.ReadEvent(App);
  var MaybeEvent: Option of Event := Application.ReadEventTimeout(App, 16);
  var Pending: Option of Event := Application.PollEvent(App);
  Application.RequestRedraw(App);
  var NeedsRedraw: boolean := Application.RedrawPending(App);
  var IsResize: boolean := Ev.kind = EventKind.Resize;
  var IsSpace: boolean := Ev.key.kind = KeyKind.Space;
  var Width: integer := Screen.width;
  var Height: integer := Ev.size.height;
  Application.Close(App)
end.",
    );
}

#[test]
fn std_tui_fully_qualified_call_works_without_uses_clause() {
    check_ok(
        "\
program T;
begin
  Std.Tui.Application.Close(Std.Tui.Application.Open())
end.",
    );
}

#[test]
fn std_tui_short_name_requires_uses() {
    let errs = check_errors(
        "\
program T;
begin
  var App: Application := Application.Open()
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Unknown type")),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_application_open_wrong_arg_count() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;
begin
  Application.Open(1)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("expects 0 arguments, got 1")),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_size_unknown_field() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;
begin
  var App: Application := Application.Open();
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
fn std_tui_event_kind_unknown_member() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;
begin
  var App: Application := Application.Open();
  var Ev: Event := Application.ReadEvent(App);
  var IsCustom: boolean := Ev.kind = Std.Tui.EventKind.Custom
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("Undefined") || e.message.contains("unknown")),
        "{errs:#?}"
    );
}

#[test]
fn uses_std_tui_case_insensitive() {
    check_ok(
        "\
program T;
uses std.tui;
begin
  var App: Application := Application.Open();
  Application.Close(App)
end.",
    );
}

//! Semantic integration tests for `Std.Tui`.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

use super::{check_errors, check_ok};

fn has_removed_tui_help(error: &fpas_diagnostics::Diagnostic) -> bool {
    error.help.as_deref().is_some_and(|help| {
        help.contains("old retained Std.Tui host/view API")
            && help.contains("Application.CreateDialog")
            && help.contains("Application.OnCommand")
    })
}

#[test]
fn std_tui_exit_reason_is_not_registered() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;
begin
  var R: ExitReason := ExitReason.UserQuit
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("Unknown type `ExitReason`")),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_application_surface_is_available() {
    check_ok(
        "\
program T;
uses Std.Tui;
begin
  var App: Application := Application.Open();
  var Screen: Size := Application.Size(App);
  var Width: integer := Screen.width;
  var Height: integer := Screen.height;
  Application.Close(App)
end.",
    );
}

#[test]
fn std_tui_list_selection_surface_is_available() {
    check_ok(
        "\
program T;
uses Std.Tui;

function Bounds(X: integer; Y: integer; Width: integer; Height: integer): Rect;
begin
  return record x := X; y := Y; width := Width; height := Height; end
end;

begin
  var App: Application := Application.Open();
  var ListHandle: ListBox := Application.CreateListBox(App, Bounds(1, 1, 20, 4), ['one'], Command.Accept);
  var Selection: integer := Application.ListSelection(App, ListHandle);
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
fn std_tui_application_close_wrong_arg_count() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;
begin
  var App: Application := Application.Open();
  Application.Close(App, App)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("expects 1 arguments, got 2")),
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

#[test]
fn std_tui_old_host_dispatch_surface_is_not_registered() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;

begin
  var App: Application := Application.Open();
  Application.HostProcessNext(App, 64)
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Unknown procedure")
            && e.message.contains("Application.HostProcessNext")
            && has_removed_tui_help(e)),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_application_run_rejects_bad_handler_and_arg_count() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;
begin
    var App: Application := Application.Open();
    Application.Run(App, App)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("OnCommand handler must be")),
        "{errs:#?}"
    );

    let errs = check_errors(
        "\
program T;
uses Std.Tui;
begin
    var App: Application := Application.Open();
    Application.Run(App, App, App)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("expects 1 or 2 arguments, got 3")),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_old_host_idle_registration_is_not_registered() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;
begin
  var App: Application := Application.Open();
  Application.HostRegisterOnIdle(App, 10)
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Unknown procedure")
            && e.message.contains("Application.HostRegisterOnIdle")
            && has_removed_tui_help(e)),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_old_host_exit_registration_is_not_registered() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;
begin
  var App: Application := Application.Open();
  Application.HostRegisterOnExit(App, OnExit);
  Application.Close(App)
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Unknown procedure")
            && e.message.contains("Application.HostRegisterOnExit")
            && has_removed_tui_help(e)),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_old_host_run_loop_is_not_registered() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;
begin
  var App: Application := Application.Open();
  Application.HostRunLoop(App)
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Unknown procedure")
            && e.message.contains("Application.HostRunLoop")
            && has_removed_tui_help(e)),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_old_host_request_quit_is_not_registered() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;

begin
  var App: Application := Application.Open();
  Application.HostRequestQuit(App)
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Unknown procedure")
            && e.message.contains("Application.HostRequestQuit")
            && has_removed_tui_help(e)),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_old_retained_query_api_is_not_registered() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;

begin
  var App: Application := Application.Open();
  Application.QuerySceneGraph(App)
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Unknown procedure")
            && e.message.contains("Application.QuerySceneGraph")
            && has_removed_tui_help(e)),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_old_framed_dialog_api_is_not_registered() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;

begin
  var App: Application := Application.Open();
  Application.ShowFramedDialog(App, 1, 1, 1, 10, 5, 'Old', false, false, false, false, true)
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Unknown procedure")
            && e.message.contains("Application.ShowFramedDialog")
            && has_removed_tui_help(e)),
        "{errs:#?}"
    );
}

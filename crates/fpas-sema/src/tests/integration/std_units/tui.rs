//! Semantic integration tests for `Std.Tui`.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

use super::{check_errors, check_ok};

#[test]
fn std_tui_exit_reason_enum_is_available() {
    check_ok(
        "\
program T;
uses Std.Tui;
begin
  var R: ExitReason := ExitReason.UserQuit;
  var H: boolean := R = ExitReason.HostStop;
    var B: boolean := R = ExitReason.HostAndUserStop;
    var S: boolean := R = ExitReason.HostShutdown;
  Application.Close(Application.Open())
end.",
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
  Application.RequestRedraw(App);
  var Width: integer := Screen.width;
  var Height: integer := Screen.height;
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
            && e.message.contains("Application.HostProcessNext")),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_application_run_wrong_arg_count() {
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
            .any(|e| e.message.contains("expects 1 arguments, got 2")),
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
            && e.message.contains("Application.HostRegisterOnIdle")),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_old_host_exit_registration_is_not_registered() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;

procedure OnExit(App: Application; Reason: ExitReason);
begin
end;

begin
  var App: Application := Application.Open();
  Application.HostRegisterOnExit(App, OnExit);
  Application.Close(App)
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Unknown procedure")
            && e.message.contains("Application.HostRegisterOnExit")),
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
            && e.message.contains("Application.HostRunLoop")),
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
            && e.message.contains("Application.HostRequestQuit")),
        "{errs:#?}"
    );
}

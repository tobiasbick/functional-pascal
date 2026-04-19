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
  var Ev: TuiEvent := Application.ReadEvent(App);
  var MaybeEvent: Option of TuiEvent := Application.ReadEventTimeout(App, 16);
  var Pending: Option of TuiEvent := Application.PollEvent(App);
  Application.RequestRedraw(App);
  var NeedsRedraw: boolean := Application.RedrawPending(App);
  var IsResize: boolean := Ev.kind = EventKind.Resize;
  var IsSpace: boolean := Ev.key.kind = Std.Console.KeyKind.Space;
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
fn std_tui_read_event_timeout_wrong_arg_count() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;
begin
  var App: Application := Application.Open();
  var Ev: Option of TuiEvent := Application.ReadEventTimeout(App)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("expects 2 arguments, got 1")),
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
  var Ev: TuiEvent := Application.ReadEvent(App);
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

#[test]
fn std_tui_host_dispatch_surface_typechecks() {
    check_ok(
        "\
program T;
uses Std.Tui;

procedure OnPaint(App: Application);
begin
end;

procedure OnIdle(App: Application);
begin
end;

begin
  var App: Application := Application.Open();
    Application.HostRegisterOnPaint(App, OnPaint);
        Application.HostRegisterOnIdle(App, 16, OnIdle);
  var Maybe: Option of TuiEvent := Application.HostPollNext(App);
  var Tag: integer := Application.HostProcessNext(App, 64);
  var Dr: integer := Application.HostDispatchRedraw(App);
  Application.HostRequestQuit(App);
    Application.Run(App);
  Application.HostRunLoop(App, 8);
end.",
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
fn std_tui_host_register_on_idle_wrong_arg_count() {
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
        errs.iter()
            .any(|e| e.message.contains("expects 3 arguments, got 2")),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_host_register_on_idle_requires_procedure_signature() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;

function WrongOnIdle(App: Application): boolean;
begin
  return true
end;

begin
  var App: Application := Application.Open();
  Application.HostRegisterOnIdle(App, 10, WrongOnIdle)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("procedure") || e.message.contains("Type mismatch")),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_host_register_on_exit_typechecks() {
    check_ok(
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
}

#[test]
fn std_tui_host_run_loop_wrong_arg_count() {
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
        errs.iter()
            .any(|e| e.message.contains("expects 2 arguments, got 1")),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_host_register_on_exit_wrong_arg_count() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;
begin
  var App: Application := Application.Open();
  Application.HostRegisterOnExit(App)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("expects 2 arguments, got 1")),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_host_register_on_exit_requires_procedure_signature() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;

function WrongOnExit(App: Application; Reason: ExitReason): boolean;
begin
  return true
end;

begin
  var App: Application := Application.Open();
  Application.HostRegisterOnExit(App, WrongOnExit)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| { e.message.contains("procedure") || e.message.contains("Type mismatch") }),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_host_request_quit_wrong_arg_count() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;
begin
  var App: Application := Application.Open();
  Application.HostRequestQuit(App, App)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("expects 1 arguments, got 2")),
        "{errs:#?}"
    );
}

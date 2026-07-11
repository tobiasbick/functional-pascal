//! Semantic integration tests for `Std.Tui`.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

use super::{check_errors, check_ok};

fn has_removed_tui_help(error: &fpas_diagnostics::Diagnostic) -> bool {
    error.help.as_deref().is_some_and(|help| {
        help.contains("old try-1 Std.Tui host/view API")
            && help.contains("Dialog.NewModal")
            && help.contains("Application.Run(App, OnCommand)")
    })
}

#[test]
fn std_tui_legacy_value_types_are_not_registered() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;
begin
  var Id: ViewId := 0;
  var DialogResultValue: DialogResult := 0;
  var Cell: ScreenCell := 0;
  var Event: TuiEvent := 0;
  var Kind: EventKind := 0;
  var Reason: ExitReason := 0
end.",
    );
    for type_name in [
        "ViewId",
        "DialogResult",
        "ScreenCell",
        "TuiEvent",
        "EventKind",
        "ExitReason",
    ] {
        assert!(
            errs.iter()
                .any(|e| e.message.contains(&format!("Unknown type `{type_name}`"))),
            "{errs:#?}"
        );
    }
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
fn std_tui_list_box_selection_surface_is_available() {
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
  var ListHandle: ListBox := ListBox.New(Bounds(1, 1, 20, 4), ['one'], CM_OK);
  var Selection: integer := ListBox.Selection(ListHandle);
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
fn std_tui_create_dialog_api_is_not_registered() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;

function Bounds(X: integer; Y: integer; Width: integer; Height: integer): Rect;
begin
  return record x := X; y := Y; width := Width; height := Height; end
end;

begin
  var App: Application := Application.Open();
  Application.CreateDialog(App, Bounds(1, 1, 10, 5), 'Old')
end.",
    );
    assert!(
        errs.iter()
            .any(|e| (e.message.contains("Unknown function or procedure")
                || e.message.contains("Unknown procedure"))
                && e.message.contains("Application.CreateDialog")
                && has_removed_tui_help(e)),
        "{errs:#?}"
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
fn std_tui_old_redraw_and_input_helpers_are_not_registered() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;

begin
  var App: Application := Application.Open();
  Application.RequestRedraw(App);
  Application.TestSendKey(App, 27)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("Application.RequestRedraw") && has_removed_tui_help(e)),
        "{errs:#?}"
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("Application.TestSendKey") && has_removed_tui_help(e)),
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

#[test]
fn std_tui_test_click_is_available() {
    check_ok(
        "\
program T;
uses Std.Tui, Std.Test;

procedure OnCommand(App: Application; Cmd: integer);
begin
  AssertEquals(CM_QUIT, Cmd);
  Application.Quit(App)
end;

function Bounds(X: integer; Y: integer; Width: integer; Height: integer): Rect;
begin
  return record x := X; y := Y; width := Width; height := Height; end
end;

begin
  var App: Application := Application.OpenForTest(40, 12);
  var Dlg: Dialog := Dialog.NewModal(Bounds(2, 1, 24, 8), 'Demo');
  var Btn: Button := Button.New(Bounds(4, 4, 10, 2), 'Quit', CM_QUIT, false);
  Dialog.Add(Dlg, Btn);
  Test.Click(App, Btn);
  Test.InjectCommand(App, CM_QUIT);
  Application.Run(App, OnCommand);
  Application.CloseForTest(App)
end.",
    );
}

#[test]
fn std_tui_test_dispatch_menu_is_available() {
    check_ok(
        "\
program T;
uses Std.Tui, Std.Test;

procedure OnCommand(App: Application; Cmd: integer);
begin
  AssertEquals(CM_QUIT, Cmd);
  Application.Quit(App)
end;

function Bounds(X: integer; Y: integer; Width: integer; Height: integer): Rect;
begin
  return record x := X; y := Y; width := Width; height := Height; end
end;

begin
  var App: Application := Application.OpenForTest(40, 12);
  var MenuBarHandle: MenuBar := MenuBar.New(Bounds(0, 0, 40, 1), [
    record title := 'File'; items := [record text := 'Quit'; commandId := CM_QUIT; end]; end
  ]);
  Application.SetMenuBar(App, MenuBarHandle);
  Test.DispatchMenu(App, MenuBarHandle, 0, 0);
  Application.Run(App, OnCommand);
  Application.CloseForTest(App)
end.",
    );
}

#[test]
fn std_tui_test_inject_command_is_available() {
    check_ok(
        "\
program T;
uses Std.Tui;

procedure OnCommand(App: Application; Cmd: integer);
begin
  Application.Quit(App)
end;

begin
  var App: Application := Application.OpenForTest(40, 12);
  Test.InjectCommand(App, CM_QUIT);
  Application.Run(App, OnCommand);
  Application.CloseForTest(App)
end.",
    );
}

#[test]
fn std_tui_test_inject_keyboard_is_available() {
    check_ok(
        "\
program T;
uses Std.Tui;

procedure OnCommand(App: Application; Cmd: integer);
begin
  Application.Quit(App)
end;

begin
  var App: Application := Application.OpenForTest(40, 12);
  Test.InjectKeyboard(App, 283);
  Application.Run(App, OnCommand);
  Application.CloseForTest(App)
end.",
    );
}

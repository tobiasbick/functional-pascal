//! Semantic integration tests for `Std.Tui`.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

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

function Bounds(X: integer; Y: integer; Width: integer; Height: integer): TuiRect;
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
fn std_tui_application_configure_surface_is_available() {
    check_ok(
        "\
program T;
uses Std.Tui;

procedure OnCommand(App: Application; Cmd: integer);
begin
  if Cmd = CM_QUIT then
    Application.Quit(App)
end;

begin
  var App: Application := Application.OpenForTest(40, 12);
  var Handlers: ApplicationHandlers := record
    OnCommand := OnCommand;
    OnKey := None;
    OnMouse := None;
  end;
  Application.Configure(App, Handlers);
  Application.CloseForTest(App)
end.",
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

function Bounds(X: integer; Y: integer; Width: integer; Height: integer): TuiRect;
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

function Bounds(X: integer; Y: integer; Width: integer; Height: integer): TuiRect;
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

//! Compiler integration tests for headless native TUI testing (Phase 1–2).
//!
//! **Documentation:** `docs/pascal/std/tui/app.md`

use super::super::{compile_and_run, compile_ok};

#[test]
fn std_tui_open_for_test_pump_and_close_succeeds() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui;

procedure OnPaint(App: Application);
begin
  ClrScr()
end;

begin
  var App: Application := Application.OpenForTest(80, 25);
  var Handlers: ApplicationHandlers := record
    OnPaint := OnPaint;
  end;
  Application.Configure(App, Handlers);
  Application.TestPump(App);
  Application.CloseForTest(App);

  var Again: Application := Application.OpenForTest(40, 10);
  Application.CloseForTest(Again)
end.",
    );

    assert!(out.lines.is_empty());
}

#[test]
fn std_tui_open_for_test_is_registered() {
    compile_ok(
        "\
program T;
uses Std.Tui;
begin
  var App: Application := Application.OpenForTest(1, 1);
  Application.CloseForTest(App)
end.",
    );
}

#[test]
fn std_tui_test_send_key_and_pump_observes_on_key_pressed() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui, Std.Test;

mutable var
  mutable var Seen: boolean := false;

procedure OnPaint(App: Application);
begin
  ClrScr()
end;

function OnKeyPressed(App: Application; Key: KeyEvent): boolean;
begin
  if Key.kind = KeyKind.Escape then
  begin
    Seen := true;
    return true
  end;
  return false
end;

begin
  var App: Application := Application.OpenForTest(80, 25);
  var Handlers: ApplicationHandlers := record
    OnPaint := OnPaint;
    OnKeyPressed := Some(OnKeyPressed);
  end;
  Application.Configure(App, Handlers);
  Application.TestSendKey(
    App,
    record
      kind := KeyKind.Escape;
      ch := #27;
      shift := false;
      ctrl := false;
      alt := false;
      meta := false;
    end);
  Application.TestPump(App);
  Application.CloseForTest(App);
  AssertTrue(Seen)
end.",
    );

    assert!(out.lines.is_empty());
}

#[test]
fn std_tui_test_send_mouse_dispatches_on_command_over_desktop() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui, Std.Test;

mutable var
  mutable var CommandSeen: boolean := false;

function FileSubmenu(): array of MenuPopupItem;
begin
  return [
    record Label := 'Open...'; Shortcut := 'O'; Enabled := false; CommandId := -1; Separator := false; end,
    record Label := ''; Shortcut := #0; Enabled := false; CommandId := -1; Separator := true; end,
    record Label := 'Exit'; Shortcut := 'X'; Enabled := true; CommandId := 1; Separator := false; end
  ]
end;

function MenuItems(): array of MenuBarItem;
begin
  return [
    record
      Label := 'File'; Shortcut := 'F'; Enabled := true; CommandId := -1;
      Submenu := FileSubmenu();
    end
  ]
end;

function MenuStyle(): MenuBarStyle;
begin
  return record
    BarBg := LightGray;
    BarFg := Black;
    AccelFg := Red;
    HighlightBg := Black;
    HighlightFg := LightGray;
    DisabledFg := DarkGray;
  end
end;

procedure OnPaint(App: Application);
begin
end;

procedure OnCommand(App: Application; CommandId: integer);
begin
  if CommandId = 1 then
    CommandSeen := true
end;

begin
  var App: Application := Application.OpenForTest(80, 25);
  var MenuBar: ViewId := Application.HostCreateMenuBarView(
    App, 0, 0, 80, 1, MenuItems(), MenuStyle());
  var Desktop: ViewId := Application.HostCreateSolidFillView(
    App, 0, 1, 80, 24, Blue, None, None);
  var Handlers: ApplicationHandlers := record
    OnPaint := OnPaint;
    OnCommand := Some(OnCommand);
  end;
  Application.Configure(App, Handlers);
  Application.TestSendMouse(
    App,
    record
      kind := Std.Console.EventKind.Mouse;
      key := record kind := KeyKind.Unknown; ch := #0; shift := false; ctrl := false; alt := false; meta := false; end;
      mouse_action := MouseAction.Down;
      mouse_button := MouseButton.Left;
      mouse_x := 2;
      mouse_y := 1;
      width := 0;
      height := 0;
      text := '';
      shift := false;
      ctrl := false;
      alt := false;
      meta := false;
    end);
  Application.TestPump(App);
  Application.TestSendMouse(
    App,
    record
      kind := Std.Console.EventKind.Mouse;
      key := record kind := KeyKind.Unknown; ch := #0; shift := false; ctrl := false; alt := false; meta := false; end;
      mouse_action := MouseAction.Down;
      mouse_button := MouseButton.Left;
      mouse_x := 2;
      mouse_y := 5;
      width := 0;
      height := 0;
      text := '';
      shift := false;
      ctrl := false;
      alt := false;
      meta := false;
    end);
  Application.TestPumpUntilIdle(App);
  Application.CloseForTest(App);
  AssertTrue(CommandSeen)
end.",
    );

    assert!(out.lines.is_empty());
}

#[test]
fn std_tui_test_click_mouse_is_registered() {
    compile_ok(
        "\
program T;
uses Std.Tui;
begin
  var App: Application := Application.OpenForTest(80, 25);
  Application.TestClickMouse(App, 1, 1);
  Application.CloseForTest(App)
end.",
    );
}

#[test]
fn std_tui_test_resize_paste_and_focus_are_registered() {
    compile_ok(
        "\
program T;
uses Std.Tui;
begin
  var App: Application := Application.OpenForTest(80, 25);
  Application.TestResize(App, 40, 10);
  Application.TestPaste(App, 'hi');
  Application.TestFocus(App, true);
  Application.CloseForTest(App)
end.",
    );
}

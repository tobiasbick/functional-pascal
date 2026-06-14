//! Compiler integration tests for native TUI query intrinsics (Phase 3–4).
//!
//! **Documentation:** `docs/pascal/std/tui-app.md`, `docs/future/tui-tests-fpas/README.md`

use super::super::{compile_and_run, compile_ok};

#[test]
fn std_tui_query_screen_size_line_and_cell_are_registered() {
    compile_ok(
        "\
program T;
uses Std.Console, Std.Tui;
begin
  var App: Application := Application.OpenForTest(80, 25);
  var ScreenSize: Size := Application.QueryScreenSize(App);
  var Line: string := Application.QueryScreenLine(App, 1);
  var Cell: ScreenCell := Application.QueryScreenCell(App, 1, 1);
  var Roots: array of integer := Application.QueryRootViews(App);
  Application.CloseForTest(App)
end.",
    );
}

#[test]
fn std_tui_query_screen_reads_painted_text_and_colors() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui, Std.Test;

procedure OnPaint(App: Application);
begin
  ClrScr();
  GotoXY(1, 1);
  Write('Hi');
  TextColor(Red);
  Write('!')
end;

begin
  var App: Application := Application.OpenForTest(80, 25);
  var Handlers: ApplicationHandlers := record
    OnPaint := OnPaint;
  end;
  Application.Configure(App, Handlers);
  Application.RequestRedraw(App);
  Application.TestPump(App);

  var ScreenSize: Size := Application.QueryScreenSize(App);
  AssertEquals(80, ScreenSize.width);
  AssertEquals(25, ScreenSize.height);

  var Bang: ScreenCell := Application.QueryScreenCell(App, 3, 1);
  AssertTrue(Bang.ch = '!');
  AssertEquals(Red, Bang.fg);

  Application.CloseForTest(App)
end.",
    );

    assert!(out.lines.is_empty());
}

#[test]
fn std_tui_query_root_views_lists_registered_roots_in_order() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui, Std.Test;

function FileSubmenu(): array of MenuPopupItem;
begin
  return [
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

begin
  var App: Application := Application.OpenForTest(80, 25);
  var MenuBar: integer := Application.HostCreateMenuBarView(
    App, 0, 0, 80, 1, MenuItems(), MenuStyle());
  var Desktop: integer := Application.HostCreateSolidFillView(
    App, 0, 1, 80, 24, Blue, None, None);
  var Roots: array of integer := Application.QueryRootViews(App);
  AssertEquals(2, Std.Array.Length(Roots));
  AssertEquals(MenuBar, Roots[0]);
  AssertEquals(Desktop, Roots[1]);
  Application.CloseForTest(App)
end.",
    );

    assert!(out.lines.is_empty());
}

#[test]
fn std_tui_query_screen_cell_reads_menu_bar_accel_color() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui, Std.Test;

function FileSubmenu(): array of MenuPopupItem;
begin
  return [
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

begin
  var App: Application := Application.OpenForTest(80, 25);
  var MenuBar: integer := Application.HostCreateMenuBarView(
    App, 0, 0, 80, 1, MenuItems(), MenuStyle());
  var Handlers: ApplicationHandlers := record
    OnPaint := OnPaint;
  end;
  Application.Configure(App, Handlers);
  Application.TestPump(App);

  var Accel: ScreenCell := Application.QueryScreenCell(App, 2, 1);
  AssertTrue(Accel.ch = 'F');
  AssertEquals(Red, Accel.fg);
  AssertEquals(LightGray, Accel.bg);

  Application.CloseForTest(App)
end.",
    );

    assert!(out.lines.is_empty());
}

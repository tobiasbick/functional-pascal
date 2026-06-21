//! Compiler integration tests for native TUI query intrinsics (Phase 3–4).
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`

use super::super::{compile_and_run, compile_ok};

#[test]
fn std_tui_query_screen_size_line_and_cell_are_registered() {
    compile_ok(
        "\
program T;
uses Std.Console, Std.Tui;
begin
  var App: Application := Application.OpenForTest(80, 25);
  var V: ViewId := Application.HostRegisterView(App, 0, 0, 1, 1);
  var ScreenSize: Size := Application.QueryScreenSize(App);
  var Line: string := Application.QueryScreenLine(App, 1);
  var Cell: ScreenCell := Application.QueryScreenCell(App, 1, 1);
  var Roots: array of ViewId := Application.QueryRootViews(App);
  var ViewRect: Rect := Application.QueryViewRect(App, V);
  var Parent: Option of ViewId := Application.QueryViewParent(App, V);
  var Children: array of ViewId := Application.QueryViewChildren(App, V);
  var ViewStateSnapshot: ViewState := Application.QueryViewState(App, V);
  var ViewOptionsSnapshot: ViewOptions := Application.QueryViewOptions(App, V);
  var Resolved: ResolvedView := Application.QueryResolvedView(App, V);
  var Kind: ViewKind := Application.QueryViewKind(App, V);
  var Scene: array of ViewSnapshot := Application.QuerySceneGraph(App);
  var MenuState: MenuBarState := Application.QueryMenuBarState(App, V);
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
  var MenuBar: ViewId := Application.HostCreateMenuBarView(
    App, 0, 0, 80, 1, MenuItems(), MenuStyle());
  var Desktop: ViewId := Application.HostCreateSolidFillView(
    App, 0, 1, 80, 24, Blue, None, None);
  var Roots: array of ViewId := Application.QueryRootViews(App);
  AssertEquals(2, Std.Array.Length(Roots));
  AssertTrue(MenuBar = Roots[0]);
  AssertTrue(Desktop = Roots[1]);
  Application.CloseForTest(App)
end.",
    );

    assert!(out.lines.is_empty());
}

#[test]
fn std_tui_query_view_rect_parent_and_children_reflect_tree() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui, Std.Test, Std.Array;

mutable var
  mutable var RootHasParent: boolean := false;

begin
  var App: Application := Application.OpenForTest(80, 25);
  var Parent: ViewId := Application.HostRegisterView(App, 0, 0, 40, 20);
  var First: ViewId := Application.HostRegisterView(App, 1, 1, 10, 5);
  var Second: ViewId := Application.HostRegisterView(App, 12, 1, 10, 5);
  Application.HostSetViewParent(App, First, Some(Parent));
  Application.HostSetViewParent(App, Second, Some(Parent));

  var Bounds: Rect := Application.QueryViewRect(App, Parent);
  AssertEquals(0, Bounds.x);
  AssertEquals(0, Bounds.y);
  AssertEquals(40, Bounds.width);
  AssertEquals(20, Bounds.height);

  var ParentOfFirst: Option of ViewId := Application.QueryViewParent(App, First);
  case ParentOfFirst of
    Some(P): AssertTrue(P = Parent);
    None: Std.Test.Fail('expected parent')
  end;

  var ParentOfRoot: Option of ViewId := Application.QueryViewParent(App, Parent);
  RootHasParent := false;
  case ParentOfRoot of
    None: RootHasParent := false;
    Some(_): RootHasParent := true
  end;
  AssertFalse(RootHasParent);

  var Kids: array of ViewId := Application.QueryViewChildren(App, Parent);
  AssertEquals(2, Std.Array.Length(Kids));
  AssertTrue(First = Kids[0]);
  AssertTrue(Second = Kids[1]);

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
  var MenuBar: ViewId := Application.HostCreateMenuBarView(
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

#[test]
fn std_tui_query_menu_bar_state_reads_initial_snapshot() {
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
  var MenuBar: ViewId := Application.HostCreateMenuBarView(
    App, 0, 0, 80, 1, MenuItems(), MenuStyle());

  var Bounds: Rect := Application.QueryViewRect(App, MenuBar);
  AssertEquals(0, Bounds.x);
  AssertEquals(80, Bounds.width);

  var State: MenuBarState := Application.QueryMenuBarState(App, MenuBar);
  AssertFalse(State.menuActive);
  AssertEquals(-1, State.hoveredIndex);
  AssertFalse(State.submenuOpen);

  Application.CloseForTest(App)
end.",
    );

    assert!(out.lines.is_empty());
}

#[test]
fn std_tui_query_menu_bar_state_reflects_submenu_after_alt_shortcut() {
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
  var MenuBar: ViewId := Application.HostCreateMenuBarView(
    App, 0, 0, 80, 1, MenuItems(), MenuStyle());
  var Key: KeyEvent := record
    kind := KeyKind.Character;
    ch := 'f';
    shift := false;
    ctrl := false;
    alt := true;
    meta := false;
  end;
  Application.TestSendKey(App, Key);
  Application.TestPump(App);

  var State: MenuBarState := Application.QueryMenuBarState(App, MenuBar);
  AssertTrue(State.menuActive);
  AssertEquals(0, State.hoveredIndex);
  AssertTrue(State.submenuOpen);
  AssertEquals(0, State.submenuBarIndex);
  AssertEquals(0, State.selectedEntry);

  Application.CloseForTest(App)
end.",
    );

    assert!(out.lines.is_empty());
}

#[test]
fn std_tui_view_id_equality_compares_opaque_handles() {
    let out = compile_and_run(
        "\
program T;
uses Std.Tui, Std.Test;

begin
  var App: Application := Application.OpenForTest(80, 25);
  var A: ViewId := Application.HostRegisterView(App, 0, 0, 10, 5);
  var B: ViewId := Application.HostRegisterView(App, 10, 0, 10, 5);
  AssertTrue(A = A);
  AssertFalse(A = B);
  Application.CloseForTest(App)
end.",
    );

    assert!(out.lines.is_empty());
}

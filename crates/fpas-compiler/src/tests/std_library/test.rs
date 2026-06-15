//! Compiler integration tests for `Std.Test`.
//!
//! **Documentation:** `docs/pascal/std/test.md` (from the repository root).

use super::super::{compile_and_run, compile_ok, compile_run_error};
use fpas_diagnostics::codes::RUNTIME_TEST_ASSERTION_FAILED;

#[test]
fn std_test_assert_equals_passes() {
    compile_ok(
        "\
program T;
uses Std.Test;
begin
  AssertEquals(4, 2 + 2);
  AssertTrue(1 + 1 = 2);
  AssertFalse(1 = 2)
end.",
    );
}

#[test]
fn std_test_assert_equals_failure_reports_code() {
    let err = compile_run_error(
        "\
program T;
uses Std.Test;
begin
  AssertEquals(4, 5)
end.",
    );
    assert_eq!(err.code, RUNTIME_TEST_ASSERTION_FAILED);
    assert!(
        err.message.contains("expected 4, got 5"),
        "message={}",
        err.message
    );
}

#[test]
fn std_test_assert_true_failure() {
    let err = compile_run_error(
        "\
program T;
uses Std.Test;
begin
  AssertTrue(false)
end.",
    );
    assert_eq!(err.code, RUNTIME_TEST_ASSERTION_FAILED);
    assert!(
        err.message.contains("expected true, got false"),
        "message={}",
        err.message
    );
}

#[test]
fn std_test_fail_with_message() {
    let err = compile_run_error(
        "\
program T;
uses Std.Test;
begin
  Fail('boom')
end.",
    );
    assert_eq!(err.code, RUNTIME_TEST_ASSERTION_FAILED);
    assert!(err.message.contains("boom"), "message={}", err.message);
}

#[test]
fn std_test_assert_equals_string_passes() {
    compile_ok(
        "\
program T;
uses Std.Test;
begin
  AssertEquals('hello', 'hel' + 'lo')
end.",
    );
}

#[test]
fn std_test_assert_equals_boolean_passes() {
    compile_ok(
        "\
program T;
uses Std.Test;
begin
  AssertEquals(true, 1 = 1)
end.",
    );
}

#[test]
fn std_test_assert_equals_real_passes() {
    compile_ok(
        "\
program T;
uses Std.Test;
begin
  AssertEquals(1.5, 3.0 / 2.0)
end.",
    );
}

#[test]
fn std_test_assert_equals_string_failure_reports_values() {
    let err = compile_run_error(
        "\
program T;
uses Std.Test;
begin
  AssertEquals('want', 'got')
end.",
    );
    assert_eq!(err.code, RUNTIME_TEST_ASSERTION_FAILED);
    assert!(
        err.message.contains("expected 'want', got 'got'"),
        "message={}",
        err.message
    );
}

#[test]
fn std_test_skip_does_not_fail() {
    compile_and_run(
        "\
program T;
uses Std.Test;
begin
  Skip('later')
end.",
    );
}

#[test]
fn std_test_assert_screen_line_passes_and_fails() {
    compile_ok(
        "\
program T;
uses Std.Console, Std.Tui, Std.Test;

procedure OnPaint(App: Application);
begin
  ClrScr();
  GotoXY(1, 1);
  Write('Hi')
end;

begin
  var App: Application := Application.OpenForTest(80, 25);
  var Handlers: ApplicationHandlers := record
    OnPaint := OnPaint;
  end;
  Application.Configure(App, Handlers);
  Application.RequestRedraw(App);
  Application.TestPump(App);
  AssertScreenLine('Hi', 1);
  Application.CloseForTest(App)
end.",
    );

    let err = compile_run_error(
        "\
program T;
uses Std.Console, Std.Tui, Std.Test;

procedure OnPaint(App: Application);
begin
  ClrScr();
  GotoXY(1, 1);
  Write('Hi')
end;

begin
  var App: Application := Application.OpenForTest(80, 25);
  var Handlers: ApplicationHandlers := record
    OnPaint := OnPaint;
  end;
  Application.Configure(App, Handlers);
  Application.RequestRedraw(App);
  Application.TestPump(App);
  AssertScreenLine('Bye', 1);
  Application.CloseForTest(App)
end.",
    );
    assert_eq!(err.code, RUNTIME_TEST_ASSERTION_FAILED);
    assert!(
        err.message.contains("expected 'Bye', got 'Hi'"),
        "message={}",
        err.message
    );
}

#[test]
fn std_test_assert_screen_line_passes_at_runtime() {
    compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui, Std.Test;

procedure OnPaint(App: Application);
begin
  ClrScr();
  GotoXY(1, 1);
  Write('Hi')
end;

begin
  var App: Application := Application.OpenForTest(80, 25);
  var Handlers: ApplicationHandlers := record
    OnPaint := OnPaint;
  end;
  Application.Configure(App, Handlers);
  Application.RequestRedraw(App);
  Application.TestPump(App);
  AssertScreenLine('Hi', 1);
  Application.CloseForTest(App)
end.",
    );
}

#[test]
fn std_test_assert_screen_cell_passes_and_fails() {
    compile_ok(
        "\
program T;
uses Std.Console, Std.Tui, Std.Test;

procedure OnPaint(App: Application);
begin
  ClrScr();
  GotoXY(1, 1);
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
  AssertScreenCell(1, 1, '!', Red, Black);
  Application.CloseForTest(App)
end.",
    );

    let err = compile_run_error(
        "\
program T;
uses Std.Console, Std.Tui, Std.Test;

procedure OnPaint(App: Application);
begin
  ClrScr();
  GotoXY(1, 1);
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
  AssertScreenCell(1, 1, '!', Blue, Black);
  Application.CloseForTest(App)
end.",
    );
    assert_eq!(err.code, RUNTIME_TEST_ASSERTION_FAILED);
    assert!(
        err.message.contains("expected 1, got 4") || err.message.contains("expected 4, got 1"),
        "message={}",
        err.message
    );
}

#[test]
fn std_test_assert_view_rect_passes_and_fails() {
    compile_ok(
        "\
program T;
uses Std.Console, Std.Tui, Std.Test;

function MenuItems(): array of MenuBarItem;
begin
  return [
    record
      Label := 'File'; Shortcut := 'F'; Enabled := true; CommandId := -1;
      Submenu := [];
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
  AssertViewRect(App, MenuBar, 0, 0, 80, 1);
  Application.CloseForTest(App)
end.",
    );

    let err = compile_run_error(
        "\
program T;
uses Std.Console, Std.Tui, Std.Test;

function MenuItems(): array of MenuBarItem;
begin
  return [
    record
      Label := 'File'; Shortcut := 'F'; Enabled := true; CommandId := -1;
      Submenu := [];
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
  AssertViewRect(App, MenuBar, 0, 0, 40, 1);
  Application.CloseForTest(App)
end.",
    );
    assert_eq!(err.code, RUNTIME_TEST_ASSERTION_FAILED);
    assert!(
        err.message.contains("expected 40, got 80"),
        "message={}",
        err.message
    );
}

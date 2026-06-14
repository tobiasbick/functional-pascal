//! Compiler integration tests for headless native TUI testing (Phase 1).
//!
//! **Documentation:** `docs/pascal/std/tui-app.md`, `docs/future/tui-tests-fpas/README.md`

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

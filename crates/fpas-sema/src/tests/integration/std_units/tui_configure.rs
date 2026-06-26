//! Semantic integration tests for `Std.Tui.Application.Configure`.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (from the repository root).

use super::{check_errors, check_ok};

#[test]
fn std_tui_application_handlers_bundle_typechecks() {
    check_ok(
        "\
program T;
uses Std.Tui;

procedure OnPaint(App: Application);
begin
end;

function OnKeyPressed(App: Application; Key: Std.Console.KeyEvent): boolean;
begin
  return true
end;

procedure OnIdle(App: Application);
begin
end;

procedure OnExit(App: Application; Reason: ExitReason);
begin
end;

begin
  var App: Application := Application.Open();
  var Handlers: ApplicationHandlers := record
    OnPaint := Some(OnPaint);
    OnKeyPressed := Some(OnKeyPressed);
    OnIdleMilliseconds := 16;
    OnIdle := Some(OnIdle);
    OnExit := Some(OnExit);
  end;
  Application.Configure(App, Handlers);
  Application.Close(App)
end.",
    );
}

#[test]
fn std_tui_application_configure_wrong_arg_count() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;
begin
  var App: Application := Application.Open();
  Application.Configure(App)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("expects 2 arguments, got 1")),
        "{errs:#?}"
    );
}

#[test]
fn std_tui_application_handlers_allow_empty_bundle() {
    check_ok(
        "\
program T;
uses Std.Tui;
begin
  var Handlers: ApplicationHandlers := record end
end.",
    );
}

#[test]
fn std_tui_application_handlers_reject_wrong_optional_handler_type() {
    let errs = check_errors(
        "\
program T;
uses Std.Tui;

procedure OnPaint(App: Application);
begin
end;

function WrongOnExit(App: Application; Reason: ExitReason): boolean;
begin
  return true
end;

begin
  var Handlers: ApplicationHandlers := record
    OnPaint := Some(OnPaint);
    OnExit := Some(WrongOnExit);
  end
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("Type mismatch") || e.message.contains("procedure")),
        "{errs:#?}"
    );
}

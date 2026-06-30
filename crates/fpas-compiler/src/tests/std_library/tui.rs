//! Compiler integration tests for the current `Std.Tui` surface.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

use super::super::{compile_and_run, compile_err, compile_ok, compile_run_error};
use fpas_bytecode::{Intrinsic, Op, TuiIntrinsic};

fn has_removed_tui_help(error: &fpas_diagnostics::Diagnostic) -> bool {
    error.help.as_deref().is_some_and(|help| {
        help.contains("old retained Std.Tui host/view API")
            && help.contains("Application.CreateDialog")
            && help.contains("Application.OnCommand")
    })
}

#[test]
fn std_tui_open_close_and_reopen_succeeds() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui;

begin
  var First: Application := Application.Open();
  Application.Close(First);

  var Second: Application := Application.Open();
  Application.RequestRedraw(Second);
  Application.Close(Second)
end.",
    );

    assert!(out.lines.is_empty());
}

#[test]
fn std_tui_open_rejects_second_session() {
    let error = compile_run_error(
        "\
program T;
uses Std.Tui;

begin
  var First: Application := Application.Open();
  var Second: Application := Application.Open()
end.",
    );

    assert!(
        error
            .message
            .contains("cannot open a second Std.Tui session"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn std_tui_use_after_close_reports_runtime_error() {
    let error = compile_run_error(
        "\
program T;
uses Std.Tui;

begin
  var App: Application := Application.Open();
  Application.Close(App);
  Application.RequestRedraw(App)
end.",
    );

    assert!(
        error
            .message
            .contains("requires an open Std.Tui application session"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn std_tui_open_rejects_wrong_argument_count() {
    let err = compile_err(
        "\
program T;
uses Std.Tui;

begin
  var App: Application := Application.Open(1)
end.",
    );

    assert!(
        err.message.contains("expects 0 arguments"),
        "unexpected compiler error: {}",
        err.message
    );
}

#[test]
fn std_tui_old_host_api_is_not_registered() {
    let err = compile_err(
        "\
program T;
uses Std.Tui;

begin
  var App: Application := Application.Open();
  Application.HostRequestQuit(App)
end.",
    );

    assert!(
        err.message.contains("Unknown procedure")
            && err.message.contains("Application.HostRequestQuit")
            && has_removed_tui_help(&err),
        "unexpected compiler error: {}",
        err.message
    );
}

#[test]
fn std_tui_old_retained_query_api_is_not_registered() {
    let err = compile_err(
        "\
program T;
uses Std.Tui;

begin
  var App: Application := Application.Open();
  Application.QuerySceneGraph(App)
end.",
    );

    assert!(
        (err.message.contains("Unknown function or procedure")
            || err.message.contains("Unknown procedure"))
            && err.message.contains("Application.QuerySceneGraph")
            && has_removed_tui_help(&err),
        "unexpected compiler error: {}",
        err.message
    );
}

#[test]
fn std_tui_old_framed_dialog_api_is_not_registered() {
    let err = compile_err(
        "\
program T;
uses Std.Tui;

begin
  var App: Application := Application.Open();
  Application.ShowFramedDialog(App, 1, 1, 1, 10, 5, 'Old', false, false, false, false, true)
end.",
    );

    assert!(
        (err.message.contains("Unknown function or procedure")
            || err.message.contains("Unknown procedure"))
            && err.message.contains("Application.ShowFramedDialog")
            && has_removed_tui_help(&err),
        "unexpected compiler error: {}",
        err.message
    );
}

#[test]
fn std_tui_turbo_vision_command_run_spike_succeeds() {
    let out = compile_and_run(
        "\
program T;
uses Std.Tui, Std.Test;

const
  CmdOk: integer := Command.Accept;

mutable var
  mutable var SeenCommand: integer := 0;

function Bounds(X: integer; Y: integer; Width: integer; Height: integer): Rect;
begin
  return record x := X; y := Y; width := Width; height := Height; end
end;

procedure OnCommand(App: Application; CommandId: integer);
begin
  SeenCommand := CommandId;
  Application.Quit(App)
end;

begin
  var App: Application := Application.OpenForTest(40, 12);
  var DialogHandle: Dialog := Application.CreateDialog(App, Bounds(2, 2, 20, 6), 'Command');
  var OkButton: Button := Application.CreateButton(App, Bounds(4, 3, 8, 1), 'OK', CmdOk);
  Application.AddChild(App, DialogHandle, OkButton);
  Application.OnCommand(App, OnCommand);
  Application.TestClickButton(App, OkButton);
  Application.Run(App);
  AssertEquals(CmdOk, SeenCommand)
end.",
    );

    assert!(out.lines.is_empty());
}

#[test]
fn std_tui_run_lowers_to_intrinsic() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Tui;

begin
  var App: Application := Application.OpenForTest(40, 12);
  Application.Run(App)
end.",
    );

    assert!(
        chunk.code().iter().any(
            |op| matches!(op, Op::Intrinsic(code) if *code == u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRun)))
        ),
        "expected Application.Run intrinsic in generated bytecode"
    );
}

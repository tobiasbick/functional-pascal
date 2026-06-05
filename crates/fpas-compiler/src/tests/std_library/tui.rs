//! Compiler integration tests for `Std.Tui`.
//!
//! **Documentation:** `docs/pascal/std/tui.md`, `docs/pascal/std/tui-app.md` (from the repository root).

use super::super::{compile_and_run, compile_err, compile_ok, compile_run_error};
use fpas_bytecode::{Intrinsic, Op, TuiIntrinsic};

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
fn std_tui_host_process_next_returns_zero_without_events() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui;

begin
  var App: Application := Application.Open();
  Std.Console.WriteLn(Application.HostProcessNext(App, 8));
  Application.Close(App)
end.",
    );

    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn std_tui_host_dispatch_redraw_not_pending_returns_zero() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui;

begin
  var App: Application := Application.Open();
  Std.Console.WriteLn(Application.HostDispatchRedraw(App));
  Application.Close(App)
end.",
    );

    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn std_tui_host_request_quit_from_on_paint_stops_host_run_loop() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui;

procedure OnPaint(App: Application);
begin
  Application.HostRequestQuit(App)
end;

begin
  var App: Application := Application.Open();
  Application.RequestRedraw(App);
  Application.HostRegisterOnPaint(App, OnPaint);
  Application.HostRunLoop(App, 100);
  Application.Close(App)
end.",
    );

    assert!(out.lines.is_empty());
}

#[test]
fn std_tui_run_invokes_on_idle_after_timeout() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui;

procedure OnPaint(App: Application);
begin
  Std.Console.WriteLn('paint')
end;

procedure OnIdle(App: Application);
begin
  Std.Console.WriteLn('idle');
  Application.HostRequestQuit(App)
end;

begin
  var App: Application := Application.Open();
  Application.HostRegisterOnPaint(App, OnPaint);
  Application.HostRegisterOnIdle(App, 1, OnIdle);
  Application.Run(App)
end.",
    );

    assert_eq!(out.lines, vec!["paint", "idle"]);
}

#[test]
fn std_tui_run_does_not_invoke_on_idle_when_interval_is_zero() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui;

procedure OnPaint(App: Application);
begin
  Std.Console.WriteLn('paint');
  Application.HostRequestQuit(App)
end;

procedure OnIdle(App: Application);
begin
  Std.Console.WriteLn('idle')
end;

begin
  var App: Application := Application.Open();
  Application.HostRegisterOnPaint(App, OnPaint);
  Application.HostRegisterOnIdle(App, 0, OnIdle);
  Application.Run(App)
end.",
    );

    assert_eq!(out.lines, vec!["paint"]);
}

#[test]
fn std_tui_run_requests_initial_paint_invokes_on_exit_and_closes_app() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui;

procedure OnPaint(App: Application);
begin
  Std.Console.WriteLn('paint');
  Application.HostRequestQuit(App)
end;

procedure OnExit(App: Application; Reason: ExitReason);
begin
  Std.Console.WriteLn(Reason)
end;

begin
  var App: Application := Application.Open();
  Application.HostRegisterOnPaint(App, OnPaint);
  Application.HostRegisterOnExit(App, OnExit);
  Application.Run(App)
end.",
    );

    assert_eq!(out.lines, vec!["paint", "Std.Tui.ExitReason.UserQuit"]);
}

#[test]
fn std_tui_run_reports_host_stop_when_close_happens_inside_handler() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui;

procedure OnPaint(App: Application);
begin
  Application.Close(App)
end;

procedure OnExit(App: Application; Reason: ExitReason);
begin
  Std.Console.WriteLn(Reason)
end;

begin
  var App: Application := Application.Open();
  Application.HostRegisterOnPaint(App, OnPaint);
  Application.HostRegisterOnExit(App, OnExit);
  Application.Run(App)
end.",
    );

    assert_eq!(out.lines, vec!["Std.Tui.ExitReason.HostStop"]);
}

#[test]
fn std_tui_run_reports_host_and_user_stop_when_close_and_quit_both_happen() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui;

procedure OnPaint(App: Application);
begin
  Application.Close(App);
  Application.HostRequestQuit(App)
end;

procedure OnExit(App: Application; Reason: ExitReason);
begin
  Std.Console.WriteLn(Reason)
end;

begin
  var App: Application := Application.Open();
  Application.HostRegisterOnPaint(App, OnPaint);
  Application.HostRegisterOnExit(App, OnExit);
  Application.Run(App)
end.",
    );

    assert_eq!(out.lines, vec!["Std.Tui.ExitReason.HostAndUserStop"]);
}

#[test]
fn std_tui_run_reports_host_shutdown_when_concurrent_task_failure_requests_vm_shutdown() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Console, Std.Tui;

procedure Crash();
begin
  panic('boom')
end;

procedure OnPaint(App: Application);
begin
  Std.Console.WriteLn('paint');
  go Crash()
end;

procedure OnExit(App: Application; Reason: ExitReason);
begin
  Std.Console.WriteLn(Reason)
end;

begin
  var App: Application := Application.Open();
  Application.HostRegisterOnPaint(App, OnPaint);
  Application.HostRegisterOnExit(App, OnExit);
  Application.Run(App)
end.",
    );

    let mut vm = fpas_vm::Vm::new(chunk);
    let error = vm
        .run()
        .expect_err("expected VM shutdown after spawned panic");
    assert!(
        error
            .message
            .contains("Execution aborted: a concurrent task failed"),
        "unexpected runtime error: {}",
        error.message
    );
    assert_eq!(
        vm.output().lines,
        vec!["paint", "Std.Tui.ExitReason.HostShutdown"],
    );
}

#[test]
fn std_tui_run_requires_registered_on_paint_handler() {
    let error = compile_run_error(
        "\
program T;
uses Std.Tui;

begin
  var App: Application := Application.Open();
  Application.Run(App)
end.",
    );

    assert!(
        error
            .message
            .contains("Application.Run(App) requires a registered OnPaint handler"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn std_tui_run_auto_close_rejects_second_close() {
    let error = compile_run_error(
        "\
program T;
uses Std.Tui;

procedure OnPaint(App: Application);
begin
  Application.HostRequestQuit(App)
end;

begin
  var App: Application := Application.Open();
  Application.HostRegisterOnPaint(App, OnPaint);
  Application.Run(App);
  Application.Close(App)
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
fn std_tui_host_register_on_paint_and_dispatch_redraw_runs_handler() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui;

procedure OnPaint(App: Application);
begin
  Std.Console.WriteLn('p')
end;

begin
  var App: Application := Application.Open();
  Application.RequestRedraw(App);
  Application.HostRegisterOnPaint(App, OnPaint);
  Std.Console.WriteLn(Application.HostDispatchRedraw(App));
  Application.Close(App)
end.",
    );

    assert_eq!(out.lines, vec!["p", "5"]);
}

#[test]
fn std_tui_run_lowers_to_intrinsic() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Tui;

procedure OnPaint(App: Application);
begin
  Application.HostRequestQuit(App)
end;

begin
  var App: Application := Application.Open();
  Application.HostRegisterOnPaint(App, OnPaint);
  Application.Run(App)
end.",
    );

    assert!(
        chunk.code.iter().any(
            |op| matches!(op, Op::Intrinsic(code) if *code == u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRun)))
        ),
        "expected Application.Run intrinsic in generated bytecode"
    );
}

#[test]
fn std_tui_host_register_on_exit_lowers_to_intrinsic() {
    let chunk = compile_ok(
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

    assert!(
    chunk
      .code
      .iter()
      .any(|op| matches!(op, Op::Intrinsic(code) if *code == u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnExit)))),
    "expected HostRegisterOnExit intrinsic in generated bytecode"
  );
}

#[test]
fn std_tui_host_register_on_idle_lowers_to_intrinsic() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Tui;

procedure OnIdle(App: Application);
begin
end;

begin
  var App: Application := Application.Open();
  Application.HostRegisterOnIdle(App, 10, OnIdle);
  Application.Close(App)
end.",
    );

    assert!(
        chunk.code.iter().any(
            |op| matches!(op, Op::Intrinsic(code) if *code == u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnIdle)))
        ),
        "expected HostRegisterOnIdle intrinsic in generated bytecode"
    );
}

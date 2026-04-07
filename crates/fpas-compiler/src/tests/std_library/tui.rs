use super::super::{compile_and_run, compile_err, compile_ok, compile_run_error};
use fpas_std::{ConsoleEvent, ConsoleKeyEvent, key_event::key_kind_index};

fn compile_run_with_console_events(source: &str, events: &[ConsoleEvent]) -> fpas_vm::VmOutput {
    let chunk = compile_ok(source);
    let mut vm = fpas_vm::Vm::new(chunk);
    for event in events {
        vm.push_console_event(event.clone());
    }
    vm.run().expect("VM should not error");
    vm.output().clone()
}

#[test]
fn std_tui_redraw_pending_is_consumed_once() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Tui;

begin
  var App: Application := Application.Open();
  Application.RequestRedraw(App);
  Std.Console.WriteLn(Application.RedrawPending(App));
  Std.Console.WriteLn(Application.RedrawPending(App));
  Application.Close(App)
end.",
    );

    assert_eq!(out.lines, vec!["true", "false"]);
}

#[test]
fn std_tui_poll_event_maps_resize_and_key_events() {
    let out = compile_run_with_console_events(
        "\
program T;
uses Std.Console, Std.Tui;

begin
  var App: Application := Application.Open();

  case Application.PollEvent(App) of
    Some(E):
      begin
        var CurrentSize: Size := Application.Size(App);
        Std.Console.WriteLn(E.kind = Std.Tui.EventKind.Resize);
        Std.Console.WriteLn(E.size.width);
        Std.Console.WriteLn(CurrentSize.width)
      end;
    None:
      Std.Console.WriteLn('missing resize')
  end;

  case Application.PollEvent(App) of
    Some(E):
      begin
        Std.Console.WriteLn(E.kind = Std.Tui.EventKind.Key);
        Std.Console.WriteLn(E.key.kind = Std.Tui.KeyKind.Space)
      end;
    None:
      Std.Console.WriteLn('missing key')
  end;

  Application.Close(App)
end.",
        &[
            ConsoleEvent::resize(120, 40),
            ConsoleEvent::key(ConsoleKeyEvent::new(
                key_kind_index("Space"),
                ' ',
                false,
                false,
                false,
                false,
            )),
        ],
    );

    assert_eq!(out.lines, vec!["true", "120", "120", "true", "true"]);
}

#[test]
fn std_tui_read_event_timeout_returns_none_without_events() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Option, Std.Tui;

begin
  var App: Application := Application.Open();
  Std.Console.WriteLn(Std.Option.IsNone(Application.ReadEventTimeout(App, 0)));
  Application.Close(App)
end.",
    );

    assert_eq!(out.lines, vec!["true"]);
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
  Std.Console.WriteLn(Application.RedrawPending(Second));
  Application.Close(Second)
end.",
    );

    assert_eq!(out.lines, vec!["true"]);
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
fn std_tui_poll_event_skips_unsupported_console_events() {
    let out = compile_run_with_console_events(
        "\
program T;
uses Std.Console, Std.Tui;

begin
  var App: Application := Application.Open();

  case Application.PollEvent(App) of
    Some(E):
      Std.Console.WriteLn(E.kind = Std.Tui.EventKind.Key);
    None:
      Std.Console.WriteLn('none')
  end;

  Application.Close(App)
end.",
        &[
            ConsoleEvent::focus_gained(),
            ConsoleEvent::paste("ignored".to_string()),
            ConsoleEvent::key(ConsoleKeyEvent::new(
                key_kind_index("Space"),
                ' ',
                false,
                false,
                false,
                false,
            )),
        ],
    );

    assert_eq!(out.lines, vec!["true"]);
}

#[test]
fn std_tui_read_event_timeout_skips_unsupported_events_before_resize() {
    let out = compile_run_with_console_events(
        "\
program T;
uses Std.Console, Std.Tui;

begin
  var App: Application := Application.Open();

  case Application.ReadEventTimeout(App, 50) of
    Some(E):
      begin
        Std.Console.WriteLn(E.kind = Std.Tui.EventKind.Resize);
        Std.Console.WriteLn(E.size.width)
      end;
    None:
      Std.Console.WriteLn('none')
  end;

  Application.Close(App)
end.",
        &[ConsoleEvent::focus_gained(), ConsoleEvent::resize(77, 25)],
    );

    assert_eq!(out.lines, vec!["true", "77"]);
}

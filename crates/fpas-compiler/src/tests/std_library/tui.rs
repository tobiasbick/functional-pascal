use super::super::{compile_and_run, compile_err, compile_ok};
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
  Std.Console.WriteLn(Application.RedrawPending(App))
end.",
    );

    assert_eq!(out.lines, vec!["true", "false"]);
}

#[test]
fn std_tui_poll_event_maps_resize_and_key_events() {
    let out = compile_run_with_console_events(
        "\
program T;
uses Std.Tui;

begin
  var App: Application := Application.Open();

  case Application.PollEvent(App) of
    Some(E):
      begin
        Std.Console.WriteLn(E.kind = Std.Tui.EventKind.Resize);
        Std.Console.WriteLn(E.size.width)
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
  end
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

    assert_eq!(out.lines, vec!["true", "120", "true", "true"]);
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

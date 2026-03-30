use super::*;
use fpas_std::ConsoleEvent;

#[test]
fn std_console_read_event_resize_updates_screen_state() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Console;
begin
  WriteLn(EventPending());
  var E: Event := ReadEvent();
  WriteLn(E.kind = EventKind.Resize);
  WriteLn(E.width);
  WriteLn(E.height);
  WriteLn(ScreenWidth());
  WriteLn(ScreenHeight())
end.",
    );
    let mut vm = fpas_vm::Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::resize(120, 40));
    vm.run().expect("VM should not error");
    assert_eq!(
        vm.output().lines,
        vec!["true", "true", "120", "40", "120", "40"]
    );
}

#[test]
fn std_console_session_procedures_compile() {
    compile_ok(
        "\
program T;
uses Std.Console;
begin
  EnableRawMode();
  DisableRawMode();
  EnterAltScreen();
  LeaveAltScreen();
  EnableMouse();
  DisableMouse();
  EnableFocus();
  DisableFocus();
  EnablePaste();
  DisablePaste()
end.",
    );
}

// ── ReadEventTimeout ──────────────────────────────────────────────────────────

#[test]
fn std_console_read_event_timeout_returns_some_when_event_queued() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Console, Std.Option;
begin
  var E: option of Event := ReadEventTimeout(100);
  WriteLn(Std.Option.IsSome(E))
end.",
    );
    let mut vm = fpas_vm::Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::resize(80, 24));
    vm.run().expect("VM should not error");
    assert_eq!(vm.output().lines, vec!["true"]);
}

#[test]
fn std_console_read_event_timeout_returns_none_in_test_mode() {
    // In test mode with no queued events, ReadEventTimeout immediately returns None.
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Option;
begin
  var E: option of Event := ReadEventTimeout(0);
  WriteLn(Std.Option.IsNone(E))
end.",
    );
    assert_eq!(out.lines, vec!["true"]);
}

#[test]
fn std_console_read_event_timeout_resize_updates_screen() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Console, Std.Option;
begin
  var E: option of Event := ReadEventTimeout(100);
  case E of
    Some(Ev): begin WriteLn(Ev.width); WriteLn(Ev.height) end;
    None: WriteLn('none')
  end
end.",
    );
    let mut vm = fpas_vm::Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::resize(100, 30));
    vm.run().expect("VM should not error");
    assert_eq!(vm.output().lines, vec!["100", "30"]);
}

// ── PollEvent ─────────────────────────────────────────────────────────────────

#[test]
fn std_console_poll_event_returns_some_when_event_queued() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Console, Std.Option;
begin
  var E: option of Event := PollEvent();
  WriteLn(Std.Option.IsSome(E))
end.",
    );
    let mut vm = fpas_vm::Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::resize(80, 24));
    vm.run().expect("VM should not error");
    assert_eq!(vm.output().lines, vec!["true"]);
}

#[test]
fn std_console_poll_event_returns_none_when_no_events() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Option;
begin
  var E: option of Event := PollEvent();
  WriteLn(Std.Option.IsNone(E))
end.",
    );
    assert_eq!(out.lines, vec!["true"]);
}

#[test]
fn std_console_poll_event_returns_option_type() {
    // Verify that PollEvent's return type is usable in case-of
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Option;
begin
  case PollEvent() of
    Some(_): WriteLn('event');
    None: WriteLn('no event')
  end
end.",
    );
    assert_eq!(out.lines, vec!["no event"]);
}

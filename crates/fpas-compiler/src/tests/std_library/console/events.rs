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

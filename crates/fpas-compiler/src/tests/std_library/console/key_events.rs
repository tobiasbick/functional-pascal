use super::*;

#[test]
fn std_console_read_key_event() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Console;
begin
  var E: Std.Console.KeyEvent := Std.Console.ReadKeyEvent();
  Std.Console.WriteLn(E.kind = Std.Console.KeyKind.Space);
  Std.Console.WriteLn(E.ch);
  Std.Console.WriteLn(E.shift);
end.",
    );
    let mut vm = fpas_vm::Vm::new(chunk);
    vm.push_key_event(ConsoleKeyEvent::new(
        key_kind_index("Space"),
        ' ',
        true,
        false,
        false,
        false,
    ));
    vm.run().expect("VM should not error");
    assert_eq!(vm.output().lines, vec!["true", " ", "true"]);
}

#[test]
fn std_console_read_key_event_character_and_fifo() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Console;
begin
  var A: Std.Console.KeyEvent := Std.Console.ReadKeyEvent();
  var B: Std.Console.KeyEvent := Std.Console.ReadKeyEvent();
  Std.Console.WriteLn(A.kind = Std.Console.KeyKind.Character);
  Std.Console.WriteLn(A.ch);
  Std.Console.WriteLn(A.ctrl);
  Std.Console.WriteLn(B.kind = Std.Console.KeyKind.Enter);
end.",
    );
    let mut vm = fpas_vm::Vm::new(chunk);
    vm.push_key_event(ConsoleKeyEvent::new(
        key_kind_index("Character"),
        'q',
        false,
        true,
        false,
        false,
    ));
    vm.push_key_event(ConsoleKeyEvent::new(
        key_kind_index("Enter"),
        '\0',
        false,
        false,
        false,
        false,
    ));
    vm.run().expect("VM should not error");
    assert_eq!(vm.output().lines, vec!["true", "q", "true", "true"]);
}

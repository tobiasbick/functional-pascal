use super::super::*;

#[test]
fn short_writeln() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  WriteLn('short')
end.",
    );
    assert_eq!(out.lines, vec!["short"]);
}

#[test]
fn short_write() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  Write('hello');
  WriteLn(' world')
end.",
    );
    assert_eq!(out.lines, vec!["hello world"]);
}

#[test]
fn short_readln() {
    let out = compile_run_with_readln(
        "\
program T;
uses Std.Console;
begin
  var S: string := ReadLn();
  WriteLn(S)
end.",
        &["hello"],
    );
    assert_eq!(out.lines, vec!["hello"]);
}

#[test]
fn short_key_event_and_enum() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Console;
begin
  var E: KeyEvent := ReadKeyEvent();
  WriteLn(E.kind = KeyKind.Space);
  WriteLn(E.ch);
  WriteLn(E.shift)
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
fn short_std_console_symbols_are_case_insensitive_in_mixed_case() {
    let chunk = compile_ok(
        "\
program T;
uses std.console;
begin
  var E: keyevent := readkeyevent();
  writeln(E.kind = keykind.space)
end.",
    );
    let mut vm = fpas_vm::Vm::new(chunk);
    vm.push_key_event(ConsoleKeyEvent::new(
        key_kind_index("Space"),
        ' ',
        false,
        false,
        false,
        false,
    ));
    vm.run().expect("VM should not error");
    assert_eq!(vm.output().lines, vec!["true"]);
}

#[test]
fn short_writeln_variadic_multiple_args() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  WriteLn('a');
  WriteLn(42);
  WriteLn(true);
  WriteLn(3.14)
end.",
    );
    assert_eq!(out.lines, vec!["a", "42", "true", "3.14"]);
}

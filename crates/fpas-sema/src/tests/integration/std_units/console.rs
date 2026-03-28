use super::{check_errors, check_ok};

#[test]
fn hello_world() {
    check_ok(
        "\
program Hello;
uses Std.Console;
begin
  Std.Console.WriteLn('Hello, World!')
end.",
    );
}

#[test]
fn std_console_read_readkey_keypressed() {
    check_ok(
        "\
program T;
uses Std.Console;
begin
  var C: char := Std.Console.Read();
  var K: char := Std.Console.ReadKey();
  var P: boolean := Std.Console.KeyPressed();
  var S: string := Std.Console.ReadLn();
  Std.Console.WriteLn(C);
  Std.Console.WriteLn(K);
  Std.Console.WriteLn(P);
  Std.Console.WriteLn(S);
end.",
    );
}

#[test]
fn std_console_read_key_event_and_fields() {
    check_ok(
        "\
program T;
uses Std.Console;
begin
  var E: Std.Console.KeyEvent := Std.Console.ReadKeyEvent();
  Std.Console.WriteLn(E.kind = Std.Console.KeyKind.Space);
  Std.Console.WriteLn(E.shift);
end.",
    );
}

#[test]
fn std_console_crt_window_and_colors() {
    check_ok(
        "\
program T;
uses Std.Console;
begin
  Window(1, 1, 40, 10);
  GotoXY(2, 3);
  TextColor(LightRed);
  TextBackground(Blue);
  CursorOff();
  CursorOn();
  Delay(0);
  ClrEol();
  ClrScr();
  var X: integer := WhereX();
  var Y: integer := WhereY();
  WriteLn(X);
  WriteLn(Y)
end.",
    );
}

#[test]
fn std_console_unified_event_api_and_session_calls() {
    check_ok(
        "\
program T;
uses Std.Console;
begin
  var Pending: boolean := EventPending();
  var E: Std.Console.Event := ReadEvent();
  WriteLn(Pending);
  WriteLn(E.kind = Std.Console.EventKind.Resize);
  WriteLn(E.mouse_button = Std.Console.MouseButton.Left);
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

#[test]
fn std_console_read_key_event_wrong_arg_count() {
    let errs = check_errors(
        "\
program T;
uses Std.Console;
begin
  Std.Console.ReadKeyEvent(1)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("expects 0 arguments, got 1")),
        "{errs:#?}"
    );
}

#[test]
fn std_console_read_key_event_wrong_arg_in_expr() {
    let errs = check_errors(
        "\
program T;
uses Std.Console;
begin
  var E: Std.Console.KeyEvent := Std.Console.ReadKeyEvent(0)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("expects 0 arguments, got 1")),
        "{errs:#?}"
    );
}

#[test]
fn std_console_key_event_unknown_field() {
    let errs = check_errors(
        "\
program T;
uses Std.Console;
begin
  var E: Std.Console.KeyEvent := Std.Console.ReadKeyEvent();
  Std.Console.WriteLn(E.not_a_field)
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("no field")),
        "{errs:#?}"
    );
}

#[test]
fn std_console_key_kind_unknown_member() {
    let errs = check_errors(
        "\
program T;
uses Std.Console;
begin
  var E: Std.Console.KeyEvent := Std.Console.ReadKeyEvent();
  Std.Console.WriteLn(E.kind = Std.Console.KeyKind.NotAKind)
end.",
    );
    assert!(
        errs.iter()
            .any(|e| { e.message.contains("Undefined") || e.message.contains("unknown") }),
        "{errs:#?}"
    );
}

#[test]
fn std_console_fully_qualified_call_works_without_uses_clause() {
    check_ok(
        "\
program T;
begin
  Std.Console.WriteLn('x')
end.",
    );
}

#[test]
fn std_console_short_name_requires_uses() {
    let errs = check_errors(
        "\
program T;
begin
  WriteLn('x')
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Unknown procedure")),
        "{errs:#?}"
    );
    let h = errs[0].help.as_deref().unwrap_or("");
    assert!(
        h.contains("uses Std.Console"),
        "hint should mention uses: {h}"
    );
}

#[test]
fn uses_std_console_case_insensitive() {
    check_ok(
        "\
program T;
uses std.console;
begin
  Std.Console.WriteLn('ok')
end.",
    );
}

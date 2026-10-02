use super::{check_errors, check_ok};

#[test]
fn hello_world() {
    check_ok(
        r#"program Hello;
uses Std.Console as Console;
begin
  Console.WriteLn('Hello, World!');
end program;"#,
    );
}

#[test]
fn std_console_read_readkey_keypressed() {
    check_ok(
        r#"program T;
uses Std.Console as Console;
begin
  var C: string := Console.ReadText();
  var K: string := Console.ReadKey();
  var P: boolean := Console.KeyPressed();
  var S: string := Console.ReadLn();
  Console.WriteLn(C);
  Console.WriteLn(K);
  Console.WriteLn(P);
  Console.WriteLn(S);
end program;"#,
    );
}

#[test]
fn std_console_read_key_event_and_fields() {
    check_ok(
        r#"program T;
uses Std.Console as Console;
begin
  var E: Console.KeyEvent := Console.ReadKeyEvent();
  Console.WriteLn(E.kind = Console.KeyKind.Space);
  Console.WriteLn(E.shift);
end program;"#,
    );
}

#[test]
fn std_console_crt_window_and_colors() {
    check_ok(
        r#"program T;
uses Std.Console as Console;
begin
  Console.Window(1, 1, 40, 10);
  Console.GotoXY(2, 3);
  Console.TextColor(Console.LightRed);
  Console.TextBackground(Console.Blue);
  Console.CursorOff();
  Console.CursorOn();
  Console.Delay(0);
  Console.ClrEol();
  Console.ClrScr();
  var X: integer := Console.WhereX();
  var Y: integer := Console.WhereY();
  Console.WriteLn(X);
  Console.WriteLn(Y);
end program;"#,
    );
}

#[test]
fn std_console_unified_event_api_and_session_calls() {
    check_ok(
        r#"program T;
uses Std.Console as Console;
begin
  var Pending: boolean := Console.EventPending();
  var E: Console.ConsoleEvent := Console.ReadEvent();
  Console.WriteLn(Pending);
  Console.WriteLn(E.kind = Console.EventKind.Resize);
  Console.WriteLn(E.mouse_button = Console.MouseButton.Left);
  Console.EnableRawMode();
  Console.DisableRawMode();
  Console.EnterAltScreen();
  Console.LeaveAltScreen();
  Console.EnableMouse();
  Console.DisableMouse();
  Console.EnableFocus();
  Console.DisableFocus();
  Console.EnablePaste();
  Console.DisablePaste();
end program;"#,
    );
}

#[test]
fn std_console_read_key_event_wrong_arg_count() {
    let errs = check_errors(
        r#"program T;
uses Std.Console as Console;
begin
  Console.ReadKeyEvent(1);
end program;"#,
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
        r#"program T;
uses Std.Console as Console;
begin
  var E: Console.KeyEvent := Console.ReadKeyEvent(0);
end program;"#,
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
        r#"program T;
uses Std.Console as Console;
begin
  var E: Console.KeyEvent := Console.ReadKeyEvent();
  Console.WriteLn(E.not_a_field);
end program;"#,
    );
    assert!(
        errs.iter().any(|e| e.message.contains("no field")),
        "{errs:#?}"
    );
}

#[test]
fn std_console_key_kind_unknown_member() {
    let errs = check_errors(
        r#"program T;
uses Std.Console as Console;
begin
  var E: Console.KeyEvent := Console.ReadKeyEvent();
  Console.WriteLn(E.kind = Console.KeyKind.NotAKind);
end program;"#,
    );
    assert!(
        errs.iter()
            .any(|e| { e.message.contains("Undefined") || e.message.contains("unknown") }),
        "{errs:#?}"
    );
}

#[test]
fn std_console_fully_qualified_call_requires_uses_clause() {
    let errs = check_errors(
        r#"program T;
begin
  Std.Console.WriteLn('x');
end program;"#,
    );
    assert_eq!(errs.len(), 1, "{errs:#?}");
    assert!(
        errs[0]
            .message
            .contains("Unit `Std.Console` is not imported"),
        "{errs:#?}"
    );
    check_ok(
        r#"program T;
uses Std.Console as Console;
begin
  Console.WriteLn('x');
end program;"#,
    );
}

#[test]
fn std_console_short_name_requires_uses() {
    let errs = check_errors(
        r#"program T;
begin
  WriteLn('x');
end program;"#,
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
        r#"program T;
uses std.console as console;
begin
  console.WriteLn('ok');
end program;"#,
    );
}

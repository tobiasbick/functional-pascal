use super::{check_errors, check_ok};

#[test]
fn hello_world() {
    check_ok(
        "program Hello;\nuses Std.Console as Console;\nbegin\n  Console.WriteLn('Hello, World!');\nend.",
    );
}

#[test]
fn std_console_read_readkey_keypressed() {
    check_ok(
        "program T;\nuses Std.Console as Console;\nbegin\n  const C: string := Console.ReadText().Unwrap();\n  const K: string := Console.ReadKey();\n  const P: boolean := Console.KeyPressed();\n  const S: string := Console.ReadLn().Unwrap();\n  Console.WriteLn(C);\n  Console.WriteLn(K);\n  Console.WriteLn(P);\n  Console.WriteLn(S);\nend.",
    );
}

#[test]
fn std_console_text_input_exposes_eof_options() {
    check_ok(
        "program T;\nuses Std.Console as Console;\nbegin\n  const Line: option of string := Console.ReadLn();\n  const Text: option of string := Console.ReadText();\n  Console.WriteLn(Line is None);\n  Console.WriteLn(Text is None);\nend.",
    );
}

#[test]
fn std_console_read_key_event_and_fields() {
    check_ok(
        "program T;\nuses Std.Console as Console;\nbegin\n  const E: Console.KeyEvent := Console.ReadKeyEvent();\n  Console.WriteLn(E.kind = Console.KeyKind.Space);\n  Console.WriteLn(E.shift);\nend.",
    );
}

#[test]
fn std_console_crt_window_and_colors() {
    check_ok(
        "program T;\nuses Std.Console as Console;\nbegin\n  Console.Window(1, 1, 40, 10);\n  Console.GotoXY(2, 3);\n  Console.TextColor(Console.LightRed);\n  Console.TextBackground(Console.Blue);\n  Console.CursorOff();\n  Console.CursorOn();\n  Console.Delay(0);\n  Console.ClrEol();\n  Console.ClrScr();\n  const X: integer := Console.WhereX();\n  const Y: integer := Console.WhereY();\n  Console.WriteLn(X);\n  Console.WriteLn(Y);\nend.",
    );
}

#[test]
fn std_console_unified_event_api_and_session_calls() {
    check_ok(
        "program T;\nuses Std.Console as Console;\nbegin\n  const Pending: boolean := Console.EventPending();\n  const E: Console.ConsoleEvent := Console.ReadEvent();\n  Console.WriteLn(Pending);\n  Console.WriteLn(E.kind = Console.EventKind.Resize);\n  Console.WriteLn(E.mouse_button = Console.MouseButton.Left);\n  Console.EnableRawMode();\n  Console.DisableRawMode();\n  Console.EnterAltScreen();\n  Console.LeaveAltScreen();\n  Console.EnableMouse();\n  Console.DisableMouse();\n  Console.EnableFocus();\n  Console.DisableFocus();\n  Console.EnablePaste();\n  Console.DisablePaste();\nend.",
    );
}

#[test]
fn std_console_read_key_event_wrong_arg_count() {
    let errs = check_errors(
        "program T;\nuses Std.Console as Console;\nbegin\n  Console.ReadKeyEvent(1);\nend.",
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
        "program T;\nuses Std.Console as Console;\nbegin\n  const E: Console.KeyEvent := Console.ReadKeyEvent(0);\nend.",
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
        "program T;\nuses Std.Console as Console;\nbegin\n  const E: Console.KeyEvent := Console.ReadKeyEvent();\n  Console.WriteLn(E.not_a_field);\nend.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("no field")),
        "{errs:#?}"
    );
}

#[test]
fn std_console_key_kind_unknown_member() {
    let errs = check_errors(
        "program T;\nuses Std.Console as Console;\nbegin\n  const E: Console.KeyEvent := Console.ReadKeyEvent();\n  Console.WriteLn(E.kind = Console.KeyKind.NotAKind);\nend.",
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
        "\
program T;
begin
  Std.Console.WriteLn('x');
end.",
    );
    assert_eq!(errs.len(), 1, "{errs:#?}");
    assert!(
        errs[0]
            .message
            .contains("Unit `Std.Console` is not imported"),
        "{errs:#?}"
    );
    check_ok("program T;\nuses Std.Console as Console;\nbegin\n  Console.WriteLn('x');\nend.");
}

#[test]
fn std_console_short_name_requires_uses() {
    let errs = check_errors(
        "\
program T;
begin
  WriteLn('x');
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
    check_ok("program T;\nuses std.console as console;\nbegin\n  console.WriteLn('ok');\nend.");
}

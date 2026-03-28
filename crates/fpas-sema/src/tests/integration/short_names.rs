use super::*;

#[test]
fn short_name_console_writeln() {
    check_ok(
        "\
program T;
uses Std.Console;
begin
  WriteLn('Hello')
end.",
    );
}

#[test]
fn short_name_console_keypressed() {
    check_ok(
        "\
program T;
uses Std.Console;
begin
  var P: boolean := KeyPressed()
end.",
    );
}

#[test]
fn short_name_console_crt_names_and_constants() {
    check_ok(
        "\
program T;
uses Std.Console;
begin
  Window(1, 1, 5, 5);
  GotoXY(1, 1);
  TextColor(Yellow);
  TextBackground(Blue);
  ClrScr();
  var X: integer := WhereX();
  var Y: integer := WhereY()
end.",
    );
}

#[test]
fn short_name_math_sqrt() {
    check_ok(
        "\
program T;
uses Std.Math;
begin
  var R: real := Sqrt(4.0)
end.",
    );
}

#[test]
fn short_name_math_pi_const() {
    check_ok(
        "\
program T;
uses Std.Math;
begin
  var R: real := Pi
end.",
    );
}

#[test]
fn short_name_conv_int_to_str() {
    check_ok(
        "\
program T;
uses Std.Conv;
begin
  var S: string := IntToStr(42)
end.",
    );
}

#[test]
fn short_name_console_key_event_type() {
    check_ok(
        "\
program T;
uses Std.Console;
begin
  var E: KeyEvent := ReadKeyEvent();
  WriteLn(E.kind = KeyKind.Space);
  WriteLn(E.shift)
end.",
    );
}

#[test]
fn short_name_mixed_with_qualified() {
    check_ok(
        "\
program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Std.Math.Sqrt(Pi))
end.",
    );
}

#[test]
fn ambiguous_length_error() {
    let errs = check_errors(
        "\
program T;
uses Std.Str, Std.Array;
begin
  var L: integer := Length('hi')
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Ambiguous")),
        "{errs:#?}"
    );
    let h = errs[0].help.as_deref().unwrap_or("");
    assert!(
        h.contains("Std.Str.Length") && h.contains("Std.Array.Length"),
        "hint should list both candidates: {h}"
    );
}

#[test]
fn ambiguous_contains_error() {
    let errs = check_errors(
        "\
program T;
uses Std.Str, Std.Array;
begin
  var B: boolean := Contains('hello', 'h')
end.",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Ambiguous")),
        "{errs:#?}"
    );
}

#[test]
fn ambiguous_fallback_to_qualified() {
    check_ok(
        "\
program T;
uses Std.Str, Std.Array;
begin
  var L: integer := Std.Str.Length('hi');
  var L2: integer := Std.Array.Length([1, 2])
end.",
    );
}

#[test]
fn no_ambiguity_single_unit() {
    check_ok(
        "\
program T;
uses Std.Str;
begin
  var L: integer := Length('hello')
end.",
    );
}

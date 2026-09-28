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
fn ambiguous_call_hint_suggests_the_method_form() {
    let errs = check_errors(
        "\
program T;
uses Std.Options, Std.Results;
begin
  var O: option of integer := Some(3);
  var X: integer := Unwrap(O)
end.",
    );
    assert_eq!(errs.len(), 1, "{errs:#?}");
    let hint = errs[0].help.as_deref().unwrap_or("");
    assert!(hint.contains("`Value.Unwrap(...)`"), "{hint}");
    // The suggested method form resolves by the receiver's type.
    check_ok(
        "\
program T;
uses Std.Options, Std.Results;
begin
  var O: option of integer := Some(3);
  var R: result of integer, string := Ok(4);
  var X: integer := O.Unwrap() + R.Unwrap()
end.",
    );
}

#[test]
fn ambiguous_length_error() {
    let errs = check_errors(
        "\
program T;
uses Std.Str, Std.Arrays;
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
        h.contains("Std.Str.Length") && h.contains("Std.Arrays.Length"),
        "hint should list both candidates: {h}"
    );
}

#[test]
fn ambiguous_length_hint_has_canonical_candidate_order() {
    let source = "\
program T;
uses Std.Str, Std.Arrays;
begin
  var L: integer := Length('hi')
end.";
    let expected = "`Length` exists in multiple imported units: Std.Arrays.Length, Std.Str.Length. Use the fully qualified name to disambiguate. Or write `Length(Value, ...)` as `Value.Length(...)`: the method form selects the routine by the type of `Value`.";

    for _ in 0..64 {
        let errors = check_errors(source);
        let help = errors
            .iter()
            .find_map(|error| error.help.as_deref())
            .expect("ambiguous name help");
        assert_eq!(help, expected);
    }
}

#[test]
fn ambiguous_contains_error() {
    let errs = check_errors(
        "\
program T;
uses Std.Str, Std.Arrays;
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
uses Std.Str, Std.Arrays;
begin
  var L: integer := Std.Str.Length('hi');
  var L2: integer := Std.Arrays.Length([1, 2])
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

/// A fully qualified name never imports its unit: the missing unit is reported, and the short
/// names of the imported units keep their meaning.
#[test]
fn qualified_std_name_without_uses_is_rejected() {
    let errs = check_errors(
        "\
program T;
uses Std.Str;
begin
  var A: array of integer := [1];
  var L1: integer := Std.Arrays.Length(A);
  var L2: integer := Length('hi')
end.",
    );
    assert_eq!(errs.len(), 1, "{errs:#?}");
    assert!(
        errs[0]
            .message
            .contains("Unit `Std.Arrays` is not imported"),
        "{errs:#?}"
    );
}

#[test]
fn qualified_names_disambiguate_imported_std_units() {
    check_ok(
        "\
program T;
uses Std.Str, Std.Arrays;
begin
  var A: array of integer := [1];
  var L1: integer := Std.Arrays.Length(A);
  var L2: integer := Std.Str.Length('hi')
end.",
    );
}

use super::*;

#[test]
fn short_name_console_writeln() {
    check_ok(
        r#"program T;
uses Std.Console as Console;
begin
  Console.WriteLn('Hello');
end program;"#,
    );
}

#[test]
fn short_name_console_keypressed() {
    check_ok(
        r#"program T;
uses Std.Console as Console;
begin
  var P: boolean := Console.KeyPressed();
end program;"#,
    );
}

#[test]
fn short_name_console_crt_names_and_constants() {
    check_ok(
        r#"program T;
uses Std.Console as Console;
begin
  Console.Window(1, 1, 5, 5);
  Console.GotoXY(1, 1);
  Console.TextColor(Console.Yellow);
  Console.TextBackground(Console.Blue);
  Console.ClrScr();
  var X: integer := Console.WhereX();
  var Y: integer := Console.WhereY();
end program;"#,
    );
}

#[test]
fn short_name_math_sqrt() {
    check_ok(
        r#"program T;
uses Std.Math as Math;
begin
  var R: real := Math.Sqrt(4.0);
end program;"#,
    );
}

#[test]
fn short_name_math_pi_const() {
    check_ok(
        r#"program T;
uses Std.Math as Math;
begin
  var R: real := Math.Pi;
end program;"#,
    );
}

#[test]
fn short_name_conv_int_to_str() {
    check_ok(
        r#"program T;
uses Std.Conv as Conv;
begin
  var S: string := Conv.IntToStr(42);
end program;"#,
    );
}

#[test]
fn short_name_console_key_event_type() {
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
fn short_name_mixed_with_qualified() {
    check_ok(
        r#"program T;
uses Std.Console as Console; uses Std.Math as Math;
begin
  Console.WriteLn(Math.Sqrt(Math.Pi));
end program;"#,
    );
}

#[test]
fn unknown_call_hint_suggests_explicit_aliases() {
    let errs = check_errors(
        "program T;\nuses Std.Options as Options; uses Std.Results as Results;\nbegin\n  var O: option of integer := Some(3);\n  var X: integer := Unwrap(O);\nend program;",
    );
    assert_eq!(errs.len(), 1, "{errs:#?}");
    let hint = errs[0].help.as_deref().unwrap_or("");
    assert!(
        hint.contains("Options.Unwrap") && hint.contains("Results.Unwrap"),
        "{hint}"
    );
    // Explicit aliases select the intended routine.
    check_ok(
        r#"program T;
uses Std.Options as Options; uses Std.Results as Results;
begin
  var O: option of integer := Some(3);
  var R: result of integer, string := Ok(4);
  var X: integer := Options.Unwrap(O) + Results.Unwrap(R);
end program;"#,
    );
}

#[test]
fn unqualified_length_error() {
    let errs = check_errors(
        "program T;\nuses Std.Str as Str; uses Std.Arrays as Arrays;\nbegin\n  var L: integer := Length('hi');\nend program;",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Unknown")),
        "{errs:#?}"
    );
    let h = errs[0].help.as_deref().unwrap_or("");
    assert!(
        h.contains("Str.Length") && h.contains("Arrays.Length"),
        "hint should list both candidates: {h}"
    );
}

#[test]
fn unqualified_length_hint_has_stable_alias_order() {
    let source = "program T;\nuses Std.Str as Str; uses Std.Arrays as Arrays;\nbegin\n  var L: integer := Length('hi');\nend program;";
    let expected = "Imports open no short names. Use `Arrays.Length` or `Str.Length`.";

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
fn unqualified_contains_error() {
    let errs = check_errors(
        "program T;\nuses Std.Str as Str; uses Std.Arrays as Arrays;\nbegin\n  var B: boolean := Contains('hello', 'h');\nend program;",
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Unknown")),
        "{errs:#?}"
    );
}

#[test]
fn ambiguous_fallback_to_qualified() {
    check_ok(
        r#"program T;
uses Std.Str as Str; uses Std.Arrays as Arrays;
begin
  var L: integer := Str.Length('hi');
  var L2: integer := Arrays.Length([1, 2]);
end program;"#,
    );
}

#[test]
fn no_ambiguity_single_unit() {
    check_ok(
        r#"program T;
uses Std.Str as Str;
begin
  var L: integer := Str.Length('hello');
end program;"#,
    );
}

/// A fully qualified name never imports its unit: the missing unit is reported, and the short
/// names of the imported units keep their meaning.
#[test]
fn qualified_std_name_without_uses_is_rejected() {
    let errs = check_errors(
        r#"program T;
uses Std.Str as Str;
begin
  var A: array of integer := [1];
  var L1: integer := Std.Arrays.Length(A);
  var L2: integer := Str.Length('hi');
end program;"#,
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
        r#"program T;
uses Std.Str as Str; uses Std.Arrays as Arrays;
begin
  var A: array of integer := [1];
  var L1: integer := Arrays.Length(A);
  var L2: integer := Str.Length('hi');
end program;"#,
    );
}

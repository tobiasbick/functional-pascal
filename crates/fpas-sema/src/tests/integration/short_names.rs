use super::*;

#[test]
fn short_name_console_writeln() {
    check_ok(
        "\
program T;
uses Std.Console;
begin
  WriteLn('Hello');
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
  const P: boolean := KeyPressed();
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
  const X: integer := WhereX();
  const Y: integer := WhereY();
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
  const R: real := Sqrt(4.0);
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
  const R: real := Pi;
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
  const S: string := IntToStr(42);
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
  const E: KeyEvent := ReadKeyEvent();
  WriteLn(E.kind = KeyKind.Space);
  WriteLn(E.shift);
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
  WriteLn(Std.Math.Sqrt(Pi));
end.",
    );
}

#[test]
fn ambiguous_call_hint_suggests_the_method_form() {
    let errs = check_errors(
        "program T;\n\nbegin\n  const O: option of integer := Some(3);\n  const X: integer := Unwrap(O);\nend.",
    );
    assert_eq!(errs.len(), 1, "{errs:#?}");
    let hint = errs[0].help.as_deref().unwrap_or("");
    assert!(hint.contains("`Value.Unwrap(…)`"), "{hint}");
    // The suggested method form resolves by the receiver's type.
    check_ok(
        "program T;\n\nbegin\n  const O: option of integer := Some(3);\n  const R: result of (integer, string) := Ok(4);\n  const X: integer := O.Unwrap() + R.Unwrap();\nend.",
    );
}

#[test]
fn old_length_free_call_has_native_migration_hint() {
    let errors = check_errors("program T; begin const L: integer := Length('hi'); end.");
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert!(
        errors[0]
            .help
            .as_deref()
            .is_some_and(|hint| hint.contains("Value.Length")),
        "{errors:#?}"
    );
}

#[test]
fn removed_helpers_have_stable_native_hints() {
    for (name, hint) in [
        ("Length('hi')", "Value.Length"),
        ("Contains('hi', 'h')", "Value.Contains"),
    ] {
        let errors = check_errors(&format!("program T; begin discard {name}; end."));
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert!(
            errors[0]
                .help
                .as_deref()
                .is_some_and(|help| help.contains(hint)),
            "{errors:#?}"
        );
    }
}

#[test]
fn removed_type_units_cannot_be_imported() {
    for unit in ["Str", "Arrays", "Dictionaries", "Options", "Results"] {
        let errors = check_errors(&format!("program T; uses Std.{unit}; begin end."));
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert!(errors[0].message.contains("removed"), "{errors:#?}");
    }
}

#[test]
fn type_operations_disambiguate_imported_units() {
    check_ok(
        "program T;\n\nbegin\n  const L: integer := 'hi'.Length();\n  const L2: integer := [1, 2].Length();\nend.",
    );
}

#[test]
fn no_ambiguity_single_unit() {
    check_ok("program T;\n\nbegin\n  const L: integer := 'hello'.Length();\nend.");
}

/// A fully qualified name never imports its unit: the missing unit is reported, and the short
/// names of the imported units keep their meaning.
#[test]
fn other_standard_units_still_require_imports() {
    let errors = check_errors("program T; begin const X: real := Std.Math.Sqrt(4.0); end.");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("not imported")),
        "{errors:#?}"
    );
}

#[test]
fn receiver_types_disambiguate_imported_std_units() {
    check_ok(
        "program T;\n\nbegin\n  const A: array of integer := [1];\n  const L1: integer := A.Length();\n  const L2: integer := 'hi'.Length();\nend.",
    );
}

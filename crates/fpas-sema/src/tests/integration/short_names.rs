use super::*;

#[test]
fn short_name_console_writeln() {
    check_ok("program T;\nuses Std.Console as Console;\nbegin\n  Console.WriteLn('Hello');\nend.");
}

#[test]
fn short_name_console_keypressed() {
    check_ok(
        "program T;\nuses Std.Console as Console;\nbegin\n  const P: boolean := Console.KeyPressed();\nend.",
    );
}

#[test]
fn short_name_console_crt_names_and_constants() {
    check_ok(
        "program T;\nuses Std.Console as Console;\nbegin\n  Console.Window(1, 1, 5, 5);\n  Console.GotoXY(1, 1);\n  Console.TextColor(Console.Yellow);\n  Console.TextBackground(Console.Blue);\n  Console.ClrScr();\n  const X: integer := Console.WhereX();\n  const Y: integer := Console.WhereY();\nend.",
    );
}

#[test]
fn short_name_math_sqrt() {
    check_ok("program T;\nuses Std.Math as Math;\nbegin\n  const R: real := Math.Sqrt(4.0);\nend.");
}

#[test]
fn short_name_math_pi_const() {
    check_ok("program T;\nuses Std.Math as Math;\nbegin\n  const R: real := Math.Pi;\nend.");
}

#[test]
fn formatting_and_numeric_conversion_have_separate_import_owners() {
    check_ok(
        "program T; uses sTd.TeXt.FoRmAt as Text, sTd.MaTh as Numbers; begin
        const S: string := Text.IntToStr(42);
        const R: real := Numbers.IntToReal(n := 42);
        const Convert: function(N: integer): real := Numbers.IntToReal;
        const ViaValue: real := Convert(42);
        end.",
    );
    for call in ["Text.IntToReal(42)", "Numbers.IntToStr(42)"] {
        let errors = check_errors(&format!(
            "program T; uses Std.Text.Format as Text, Std.Math as Numbers; begin discard {call}; end."
        ));
        assert_eq!(errors.len(), 1, "{call}: {errors:#?}");
        assert_eq!(errors[0].code, fpas_diagnostics::codes::SEMA_UNKNOWN_NAME);
    }
}

#[test]
fn short_name_console_key_event_type() {
    check_ok(
        "program T;\nuses Std.Console as Console;\nbegin\n  const E: Console.KeyEvent := Console.ReadKeyEvent();\n  Console.WriteLn(E.kind = Console.KeyKind.Space);\n  Console.WriteLn(E.shift);\nend.",
    );
}

#[test]
fn short_name_mixed_with_qualified() {
    check_ok(
        "program T;\nuses Std.Console as Console, Std.Math as Math;\nbegin\n  Console.WriteLn(Math.Sqrt(Math.Pi));\nend.",
    );
}

#[test]
fn required_qualification_hint_lists_each_import_and_leads_to_a_valid_call() {
    let source = "program T; uses Std.Console as Console, Std.Io.Fs as Fs; begin const Text: result of (string, Fs.FsError) := ReadText('input.txt'); end.";
    let errors = check_errors(source);
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(
        errors[0].code,
        fpas_diagnostics::codes::SEMA_UNQUALIFIED_NAME
    );
    let hint = errors[0].help.as_deref().expect("qualification hint");
    assert!(
        hint.contains("Console.ReadText") && hint.contains("Fs.ReadText"),
        "{hint}"
    );
    check_ok(&source.replace(":= ReadText(", ":= Fs.ReadText("));

    let statement_errors = check_errors(
        "program T; uses Std.Console as Console, Std.Io.Fs as Fs; begin ReadText('input.txt'); end.",
    );
    assert_eq!(statement_errors.len(), 1, "{statement_errors:#?}");
    assert_eq!(statement_errors[0].code, errors[0].code);
    assert_eq!(statement_errors[0].help, errors[0].help);
    check_ok(
        "program T; uses Std.Console as Console, Std.Io.Fs as Fs; begin discard Fs.ReadText('input.txt'); end.",
    );
}

#[test]
fn removed_unwrap_free_call_suggests_the_native_method() {
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
            .is_some_and(|hint| hint.contains("FPAS writes `'hi'.Length()`")),
        "{errors:#?}"
    );
}

#[test]
fn removed_helpers_have_stable_native_hints() {
    for (name, hint, corrected) in [
        ("Length('hi')", "'hi'.Length()", "'hi'.Length()"),
        ("Contains('hi', 'h')", "Substring in Text", "'h' in 'hi'"),
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
        check_ok(&format!("program T; begin discard {corrected}; end."));
    }
}

#[test]
fn removed_type_units_cannot_be_imported() {
    for unit in ["Str", "Arrays", "Dictionaries", "Options", "Results"] {
        let errors = check_errors(&format!("program T; uses Std.{unit} as Legacy; begin end."));
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

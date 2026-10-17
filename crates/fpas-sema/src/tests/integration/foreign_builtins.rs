//! Delphi and Pascal built-in names, type names and constructors: specific help with the written
//! arguments, and no hint for user declarations with the same names.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

use super::*;
use fpas_diagnostics::codes::{SEMA_UNKNOWN_NAME, SEMA_UNKNOWN_TYPE};

/// Asserts exactly one error with `code` whose help ends with the FPAS form `replacement`.
fn assert_help(source: &str, code: fpas_diagnostics::DiagnosticCode, replacement: &str) {
    let source = format!("program P;\n\nuses Std.Console as Console;\n\n{source}");
    let errors = check_errors(&source);
    assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
    assert_eq!(errors[0].code, code, "{source}");
    let help = errors[0].help.as_deref().unwrap_or_default();
    assert!(
        help.ends_with(&format!("FPAS writes `{replacement}`.")),
        "{source}: {help}"
    );
}

#[test]
fn pascal_builtins_name_their_fpas_form_with_the_written_arguments() {
    for (statement, replacement) in [
        ("Inc(Count);", "Count := Count + 1;"),
        ("Dec(Count, 2);", "Count := Count - 2;"),
        ("Console.WriteLn(Length(Text));", "Text.Length()"),
        ("Console.WriteLn(High(Items));", "Items.Length() - 1"),
        ("Console.WriteLn(Low(Items));", "0"),
        ("Console.WriteLn(IntToStr(Count));", "Conv.IntToStr(Count)"),
        ("Write(Text);", "Console.WriteText(Text)"),
        ("Console.Write(Text);", "Console.WriteText(Text)"),
        ("discard TryStrToInt(Text, Count);", "Parse.Int(Text)"),
    ] {
        assert_help(
            &format!(
                "begin\n  var Count: integer := 0;\n  const Text := 'abc';\n  const Items := [1, 2];\n  {statement}\n  Count := Count + Items.Length() + Text.Length();\nend."
            ),
            SEMA_UNKNOWN_NAME,
            replacement,
        );
    }
}

#[test]
fn nil_names_none() {
    assert_help(
        "begin\n  const Value: Option of integer := nil;\nend.",
        SEMA_UNKNOWN_NAME,
        "None",
    );
}

#[test]
fn foreign_type_names_name_the_fpas_type() {
    for (written, replacement) in [
        ("char", "string"),
        ("Double", "real"),
        ("Single", "real"),
        ("Extended", "real"),
        ("Int64", "integer"),
        ("AnsiString", "string"),
    ] {
        assert_help(
            &format!("begin\n  var Value: {written} := 0;\nend."),
            SEMA_UNKNOWN_TYPE,
            replacement,
        );
    }
}

#[test]
fn record_create_names_the_fields() {
    assert_help(
        "type Point = record\n  X: integer;\n  Y: integer;\nend record;\n\nbegin\n  const P := Point.Create(1, 2);\nend.",
        SEMA_UNKNOWN_NAME,
        "Point(X := 1, Y := 2)",
    );
}

#[test]
fn user_declarations_with_foreign_names_are_not_reported() {
    check_ok(
        "program P;

type Double = integer;
type Point = record
  X: integer;
  static function Create(X: integer): Point;
  begin
    return Point(X := X);
  end function;
end record;

procedure Inc(var Value: integer);
begin
  Value := Value + 1;
end procedure;

function High(Items: array of integer): integer;
begin
  return Items.Length() - 1;
end function;

begin
  var Count: Double := 0;
  Inc(var Count);
  const P := Point.Create(High([1, 2]));
  Count := Count + P.X;
end.",
    );
}

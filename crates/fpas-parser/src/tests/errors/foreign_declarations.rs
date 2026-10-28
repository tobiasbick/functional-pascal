//! Pascal, Ada and Delphi declaration habits: one diagnostic per group, recovery, and no false
//! positives.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

use super::foreign_habit_assertions::{LATER_ERROR, assert_one_habit, compilation_unit_errors};
use crate::tests::{parse_ok, parse_unit_ok};
use fpas_diagnostics::codes::{
    PARSE_EXPECTED_TOKEN, PARSE_FOREIGN_SYNTAX, PARSE_MISSING_DECLARATION_KEYWORD,
    PARSE_MISSING_IMPORT_ALIAS,
};

#[test]
fn pascal_sections_are_reported_once() {
    assert_one_habit(
        "var\n  A: integer := 1;\n  B: integer := 2;\n  C: integer := 3;",
        PARSE_MISSING_DECLARATION_KEYWORD,
        "Pascal `var` section",
        "var\n  A: integer := 1;\n  B: integer := 2;\n  C: integer := 3;",
        "var A: integer := 1; var B: integer := 2; var C: integer := 3;",
    );
    assert_one_habit(
        "var\n  A: integer;\n  B: string;",
        PARSE_MISSING_DECLARATION_KEYWORD,
        "Pascal `var` section",
        "var\n  A: integer;\n  B: string;",
        "var A: integer := 0; var B: string := '';",
    );
    assert_one_habit(
        "const\n  N = 3;\n  Name = 'x';",
        PARSE_MISSING_DECLARATION_KEYWORD,
        "Pascal `const` section",
        "const\n  N = 3;\n  Name = 'x';",
        "const N: integer := 3; const Name: string := 'x';",
    );
}

#[test]
fn single_pascal_binding_forms_name_their_fpas_form() {
    assert_one_habit(
        "const N = 3;",
        PARSE_FOREIGN_SYNTAX,
        "Pascal `const Name = Value`",
        "const N = 3;",
        "const N: integer := 3;",
    );
    assert_one_habit(
        "var I, Sum: integer := 0;",
        PARSE_MISSING_DECLARATION_KEYWORD,
        "Grouped names `I, Sum`",
        "var I, Sum: integer := 0;",
        "var I: integer := 0; var Sum: integer := 0;",
    );
    assert_one_habit(
        "var Total: integer;",
        PARSE_EXPECTED_TOKEN,
        "`Total` has no initializer",
        "var Total: integer;",
        "var Total: integer := 0;",
    );
    assert_one_habit(
        "const Numbers: array[0..2] of integer = (3, 7, 12);",
        PARSE_FOREIGN_SYNTAX,
        "Pascal bounded array type",
        "const Numbers: array[0..2] of integer = (3, 7, 12);",
        "const Numbers: array of integer := [3, 7, 12];",
    );
}

#[test]
fn ada_binding_without_keyword_reuses_its_initializer() {
    assert_one_habit(
        "X : integer := 5;",
        PARSE_MISSING_DECLARATION_KEYWORD,
        "requires its own `var` keyword",
        "X : integer := 5;",
        "var X: integer := 5;",
    );
}

#[test]
fn routine_declaration_section_becomes_local_bindings() {
    assert_one_habit(
        "procedure Q();\nvar\n  I: integer;\n  Name: string := 'x';\nbegin\n  I := 1;\nend procedure;",
        PARSE_FOREIGN_SYNTAX,
        "Pascal declaration section before `begin`",
        "var\n  I: integer;\n  Name: string := 'x';",
        "var I: integer := 0; var Name: string := 'x';",
    );
}

#[test]
fn local_grouped_binding_is_reported_once() {
    assert_one_habit(
        "procedure Q();\nbegin\n  var I, J: integer := 0;\n  I := J;\nend procedure;",
        PARSE_MISSING_DECLARATION_KEYWORD,
        "Grouped names `I, J`",
        "var I, J: integer := 0",
        "var I: integer := 0; var J: integer := 0;",
    );
}

#[test]
fn delphi_class_is_reported_once() {
    assert_one_habit(
        "type TFoo = class(TObject)\n  X: integer;\n  procedure Bar;\nend;",
        PARSE_FOREIGN_SYNTAX,
        "Delphi `class`",
        "class(TObject)\n  X: integer;\n  procedure Bar;\nend",
        "type TFoo = record … function Name(Self: TFoo): T; … end record;",
    );
}

#[test]
fn delphi_containers_in_angle_brackets_name_the_native_type() {
    assert_one_habit(
        "var L: TList<integer> := [];",
        PARSE_EXPECTED_TOKEN,
        "Type applications use `of`",
        "TList<integer>",
        "array of integer",
    );
}

#[test]
fn delphi_run_time_library_unit_names_the_std_units() {
    assert_one_habit(
        "uses SysUtils;",
        PARSE_MISSING_IMPORT_ALIAS,
        "Delphi unit `SysUtils` does not exist",
        "SysUtils",
        "uses Std.Text.Parse as Parse, Std.Text.Format as Conv;",
    );
}

#[test]
fn delphi_unit_sections_are_reported_once_and_recovered() {
    let source = "unit U;\n\ninterface\n\nuses SysUtils;\n\nfunction F(X: integer): integer;\n\nimplementation\n\nfunction F(X: integer): integer;\nbegin\n  return X;\nend function;\n\nend.";
    let errors = compilation_unit_errors(source);
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, PARSE_FOREIGN_SYNTAX);
    assert_eq!(
        errors[0].expected.as_deref(),
        Some("unit U; public function F(X: integer): integer; begin … end function; end unit;")
    );

    let source = "unit U;\n\ninterface\n\nimplementation\n\nfunction F(): integer;\nbegin\n  return 1 +;\nend function;\n\nend.";
    let errors = compilation_unit_errors(source);
    assert_eq!(errors.len(), 2, "{errors:#?}");
    assert_eq!(errors[1].span.expect("source span").line(), 9);
}

#[test]
fn valid_declarations_with_the_same_tokens_are_not_reported() {
    parse_ok(&format!(
        "program P;

type Class = integer;
type Alias = Class;
type Box of T = record
  Value: T;
end record;

const N: integer := 3;
const Numbers: array of integer := [3, 7, 12];
var Total: integer := 0;
var Items: Box of integer := Box(Value := 1);

procedure Q();
begin
  const Local := 1;
  var Copy: integer := Local;
  Copy := Copy + N;
end procedure;

{}",
        LATER_ERROR.replace("Foo(1;", "Q();")
    ));
    parse_unit_ok(
        "unit U;\n\nuses Std.Math as Math;\n\npublic const Interface: integer := 1;\n\nend unit;",
    );
}

//! Ada program structure, Ada and Pascal type declarations, and Delphi expression habits found
//! by the full blind experiment: one diagnostic each, recovery, and no false positives.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

use super::foreign_habit_assertions::{LATER_ERROR, assert_one_habit, parser_errors};
use crate::tests::parse_ok;
use fpas_diagnostics::codes::{
    PARSE_FOREIGN_SYNTAX, PARSE_INVALID_CALL_OR_ASSIGNMENT_FORM, PARSE_MISSING_DECLARATION_KEYWORD,
    PARSE_MISSING_IMPORT_ALIAS,
};

#[test]
fn ada_main_program_is_reported_once_and_recovered() {
    let source = "program Main is\n  Count : integer := 0;\nbegin\n  Count := 1;\nend Main;";
    let errors = parser_errors(source);
    let codes = errors.iter().map(|error| error.code).collect::<Vec<_>>();
    assert_eq!(
        codes,
        [PARSE_FOREIGN_SYNTAX, PARSE_MISSING_DECLARATION_KEYWORD],
        "{errors:#?}"
    );
    assert_eq!(
        errors[0].expected.as_deref(),
        Some("program Main; … begin … end.")
    );

    let errors = parser_errors("program Main is\nbegin\n  Foo(1;\nend Main;");
    assert_eq!(errors.len(), 2, "{errors:#?}");
    assert_eq!(errors[1].span.expect("source span").line(), 3);
}

#[test]
fn ada_and_pascal_type_declarations_name_the_fpas_form() {
    for (declaration, found, replacement) in [
        (
            "type Point is record\n  X : real;\nend record;",
            "Point is record\n  X : real;\nend record",
            "type Point = record … end record;",
        ),
        (
            "type Kind is (Circle, Square);",
            "Kind is (Circle, Square)",
            "type Kind = enum Circle; Square; end enum;",
        ),
        (
            "type Values is array (1 .. 5) of integer;",
            "Values is array (1 .. 5) of integer",
            "type Values = array of integer;",
        ),
        (
            "type TKind = (skCircle, skSquare);",
            "TKind = (skCircle, skSquare)",
            "type TKind = enum skCircle; skSquare; end enum;",
        ),
    ] {
        assert_one_habit(declaration, PARSE_FOREIGN_SYNTAX, "", found, replacement);
    }
}

#[test]
fn ada_constants_parameter_modes_and_aggregates_are_reported_once() {
    assert_one_habit(
        "Limit : constant integer := 10;",
        PARSE_MISSING_DECLARATION_KEYWORD,
        "requires its own `const` keyword",
        "Limit : constant integer := 10;",
        "const Limit: integer := 10;",
    );
    assert_one_habit(
        "procedure Bump(Counter : in out integer);\nbegin\n  Counter := Counter + 1;\nend procedure;",
        PARSE_FOREIGN_SYNTAX,
        "Ada parameter mode",
        "Counter : in out integer",
        "var Counter: integer",
    );
    assert_one_habit(
        "const Values: array of integer := (3, 7, 12);",
        PARSE_FOREIGN_SYNTAX,
        "Ada aggregate",
        "const Values: array of integer := (3, 7, 12);",
        "const Values: array of integer := [3, 7, 12];",
    );
}

#[test]
fn delphi_expression_habits_are_reported_once() {
    assert_one_habit(
        "procedure Q();\nbegin\n  var Names := TList<string>.Create;\nend procedure;",
        PARSE_FOREIGN_SYNTAX,
        "Delphi generic constructor",
        "TList<string>.Create",
        "[]",
    );
    assert_one_habit(
        "procedure Q();\nbegin\n  Exit;\nend procedure;",
        PARSE_INVALID_CALL_OR_ASSIGNMENT_FORM,
        "Delphi `Exit`",
        "Exit",
        "return",
    );
    assert_one_habit(
        "function F(): integer;\nbegin\n  Inc(Result);\n  return 1;\nend function;",
        PARSE_FOREIGN_SYNTAX,
        "Delphi `Result` is not an FPAS variable",
        "Result",
        "Value",
    );
    assert_one_habit(
        "uses System.SysUtils;",
        PARSE_MISSING_IMPORT_ALIAS,
        "Delphi unit `System.SysUtils`",
        "System.SysUtils",
        "uses Std.Parse as Parse, Std.Conv as Conv;",
    );
}

#[test]
fn valid_forms_with_the_same_tokens_are_not_reported() {
    parse_ok(&format!(
        "program P;

type Kind = enum
  Circle;
  Square;
end enum;

type Point = record
  X: integer;
  Y: integer;
end record;

const Origin: Point := Point(X := 0, Y := 0);

function Grouped(A: integer; B: integer): result of (integer, string);
begin
  const Sum := (A + B);
  const Same: Point := (Origin with X := 1; end with) with Y := 2; end with;
  if Sum < 0 or (Sum > 10) then
    return Error('range');
  end if;
  return Ok(Sum);
end function;

procedure Q(X: integer);
begin
  case X of
    when 1:
      null;
    else
      null;
  end case;
end procedure;

{}",
        LATER_ERROR.replace("Foo(1;", "Q(1);")
    ));
}

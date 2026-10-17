//! Delphi and Ada routine habits: one diagnostic each, recovery, and no false positives.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

use super::foreign_habit_assertions::assert_one_habit;
use crate::tests::parse_ok;
use fpas_diagnostics::codes::{PARSE_EXPECTED_TOKEN, PARSE_FOREIGN_SYNTAX};

#[test]
fn const_parameter_modifier_is_reported_once_per_parameter() {
    assert_one_habit(
        "function F(const S: string): integer;\nbegin\n  return S.Length();\nend function;",
        PARSE_FOREIGN_SYNTAX,
        "Delphi `const` parameter modifier",
        "const S: string",
        "S: string",
    );
    assert_one_habit(
        "procedure P(var A: integer; const B: integer);\nbegin\n  A := B;\nend procedure;",
        PARSE_FOREIGN_SYNTAX,
        "Delphi `const` parameter modifier",
        "const B: integer",
        "B: integer",
    );
}

#[test]
fn routine_without_parameter_list_is_recovered_as_empty_list() {
    assert_one_habit(
        "procedure Q;\nbegin\n  null;\nend procedure;",
        PARSE_EXPECTED_TOKEN,
        "Routine `Q` has no parameter list",
        "Q",
        "Q()",
    );
    assert_one_habit(
        "function F: integer;\nbegin\n  return 1;\nend function;",
        PARSE_EXPECTED_TOKEN,
        "Routine `F` has no parameter list",
        "F",
        "F()",
    );
}

#[test]
fn ada_routine_header_and_body_give_one_diagnostic() {
    assert_one_habit(
        "function F return integer is\nbegin\n  return 1;\nend F;",
        PARSE_FOREIGN_SYNTAX,
        "Ada routine header",
        "function F return integer is",
        "function F(): integer;",
    );
    assert_one_habit(
        "function Add(A: integer; B: integer) return integer is\nbegin\n  if A > B then\n    return A;\n  end if;\n  return B;\nend Add;",
        PARSE_FOREIGN_SYNTAX,
        "Ada routine header",
        "function Add(A: integer; B: integer) return integer is",
        "function Add(A: integer; B: integer): integer;",
    );
    assert_one_habit(
        "procedure Show is\nbegin\n  null;\nend Show;",
        PARSE_FOREIGN_SYNTAX,
        "Ada routine header",
        "procedure Show is",
        "procedure Show();",
    );
}

#[test]
fn ada_result_type_with_fpas_body_is_recovered() {
    assert_one_habit(
        "function F() return integer;\nbegin\n  return 1;\nend function;",
        PARSE_FOREIGN_SYNTAX,
        "Ada routine header",
        "function F() return integer",
        "function F(): integer;",
    );
}

#[test]
fn delphi_result_assignment_is_recovered_as_return() {
    assert_one_habit(
        "function F(): integer;\nbegin\n  Result := 1 + 2;\nend function;",
        PARSE_FOREIGN_SYNTAX,
        "Delphi `Result :=`",
        "Result := 1 + 2",
        "return 1 + 2",
    );
    assert_one_habit(
        "type Point = record\n  X: real;\nend record;\n\nfunction Make(A: real): Point;\nbegin\n  Result.X := A;\nend function;",
        PARSE_FOREIGN_SYNTAX,
        "Delphi `Result :=`",
        "Result.X := A",
        "return TypeName(X := A, …)",
    );
}

#[test]
fn valid_forms_with_the_same_tokens_are_not_reported() {
    parse_ok(
        "program P;

type Point = record
  X: real;
  function Scaled(Self: Point; Factor: real): Point;
  begin
    return Point(X := Self.X * Factor);
  end function;
end record;

function F(S: string): result of (integer, string);
begin
  const Length := S.Length();
  return Ok(Length);
end function;

procedure Q();
begin
  return;
end procedure;

begin
  const Callback := function(Value: integer): integer begin
    return Value;
  end function;
  discard Callback(1);
end.",
    );
}

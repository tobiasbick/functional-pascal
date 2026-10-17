//! Pascal, Ada and Delphi control-flow habits: one diagnostic per statement, recovery, and no
//! false positives.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

use super::foreign_habit_assertions::assert_one_habit;
use crate::tests::parse_ok;
use fpas_diagnostics::codes::{PARSE_EXPECTED_TOKEN, PARSE_FOREIGN_SYNTAX};

/// Wraps statements in a procedure that closes with its FPAS ending.
fn in_procedure(statements: &str) -> String {
    format!("procedure Q(X: integer);\nbegin\n  var Y: integer := 0;\n{statements}\nend procedure;")
}

#[test]
fn pascal_if_statements_are_reported_once() {
    for (statements, found, replacement) in [
        (
            "  if X > 0 then\n    Y := 1\n  else\n    Y := 2;",
            "if X > 0 then\n    Y := 1\n  else\n    Y := 2;",
            "if X > 0 then …; else …; end if;",
        ),
        (
            "  if X > 0 then begin\n    Y := 1;\n  end;",
            "if X > 0 then begin\n    Y := 1;\n  end;",
            "if X > 0 then …; end if;",
        ),
        (
            "  if X > 0 then\n  begin\n    Y := 1;\n  end\n  else\n  begin\n    Y := 2;\n  end;",
            "if X > 0 then\n  begin\n    Y := 1;\n  end\n  else\n  begin\n    Y := 2;\n  end;",
            "if X > 0 then …; else …; end if;",
        ),
    ] {
        assert_one_habit(
            &in_procedure(statements),
            PARSE_EXPECTED_TOKEN,
            "Pascal `if` statement",
            found,
            replacement,
        );
    }
}

#[test]
fn pascal_loops_are_reported_once() {
    assert_one_habit(
        &in_procedure("  while Y < X do\n  begin\n    Y := Y + 1;\n  end;"),
        PARSE_EXPECTED_TOKEN,
        "Pascal `while` statement",
        "while Y < X do\n  begin\n    Y := Y + 1;\n  end;",
        "while Y < X do …; end while;",
    );
    assert_one_habit(
        &in_procedure("  for I := 1 to X do\n    Y := Y + I;"),
        PARSE_EXPECTED_TOKEN,
        "Pascal `for` statement",
        "for I := 1 to X do\n    Y := Y + I;",
        "for I: integer := 1 to X do …; end for;",
    );
    assert_one_habit(
        &in_procedure("  for I := 1 to X do\n    Y := Y + I;\n  end for;"),
        PARSE_EXPECTED_TOKEN,
        "Pascal `for` statement",
        "for I := 1 to X do\n    Y := Y + I;\n  end for",
        "for I: integer := 1 to X do …; end for;",
    );
}

#[test]
fn ada_loops_are_skipped_and_reported_once() {
    assert_one_habit(
        &in_procedure("  for I in 1 .. X loop\n    Y := Y + I;\n  end loop;"),
        PARSE_FOREIGN_SYNTAX,
        "Ada loop",
        "for I in 1 .. X loop\n    Y := Y + I;\n  end loop",
        "for I: integer := 1 to X do …; end for;",
    );
    assert_one_habit(
        &in_procedure("  while Y < X loop\n    Y := Y + 1;\n  end loop;"),
        PARSE_FOREIGN_SYNTAX,
        "Ada loop",
        "while Y < X loop\n    Y := Y + 1;\n  end loop",
        "while Y < X do …; end while;",
    );
}

#[test]
fn pascal_case_arms_are_reported_once() {
    assert_one_habit(
        &in_procedure(
            "  case X of\n    1: Y := 1;\n    2, 3: Y := 2;\n  else\n    Y := 0;\n  end;",
        ),
        PARSE_EXPECTED_TOKEN,
        "Pascal `case` statement",
        "case X of\n    1: Y := 1;\n    2, 3: Y := 2;\n  else\n    Y := 0;\n  end",
        "case X of when 1: …; when 2, 3: …; else …; end case;",
    );
}

#[test]
fn delphi_try_blocks_are_reported_once() {
    assert_one_habit(
        &in_procedure(
            "  try\n    Y := 1;\n    if X > 0 then\n      Y := 2;\n    end if;\n  except\n    Y := 0;\n  end;",
        ),
        PARSE_FOREIGN_SYNTAX,
        "Delphi `try … except`",
        "try\n    Y := 1;\n    if X > 0 then\n      Y := 2;\n    end if;\n  except\n    Y := 0;\n  end",
        "case Operation() of when Ok(const Value): …; when Error(const Message): …; end case;",
    );
    assert_one_habit(
        &in_procedure("  try\n    Y := 1;\n  finally\n    Y := 0;\n  end;"),
        PARSE_FOREIGN_SYNTAX,
        "Delphi `try … finally`",
        "try\n    Y := 1;\n  finally\n    Y := 0;\n  end",
        "Work(); Cleanup();",
    );
}

#[test]
fn valid_control_flow_with_the_same_tokens_is_not_reported() {
    parse_ok(
        "program P;

function Half(X: integer): result of (integer, string);
begin
  if X mod 2 = 0 then
    return Ok(X div 2);
  end if;
  return Error('odd');
end function;

function Quarter(X: integer): result of (integer, string);
begin
  const H := try Half(X);
  return Half(H);
end function;

procedure Q(X: integer);
begin
  var Loop: integer := 0;
  const Except := 1;
  if X > 0 then
    begin
      Loop := 1;
    end;
    Loop := Loop + Except;
  else
    Loop := 2;
  end if;
  while Loop < X do
    Loop := Loop + 1;
  end while;
  for I: integer := 1 to X do
    Loop := Loop + I;
  end for;
  case X of
    when 1:
      Loop := 0;
    else
      null;
  end case;
end procedure;

begin
  Q(1);
  discard Quarter(8);
end.",
    );
}

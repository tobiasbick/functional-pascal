//! `Value is Pattern` tests in `if`, `elsif`, and `while` conditions.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/is-test.md`

use super::super::{check_errors, check_ok};
use fpas_diagnostics::DiagnosticCode;
use fpas_diagnostics::codes::{
    SEMA_DUPLICATE_DECLARATION, SEMA_IMPLICIT_PATTERN_BINDING, SEMA_MISPLACED_IS_TEST,
    SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME,
};

fn program(body: &str) -> String {
    format!(
        "program T;
        type Shape = enum Circle(Radius: integer); Point; end enum;
        procedure P(S: Shape; O: option of integer; R: result of (option of integer, string));
        begin
          {body}
        end procedure;
        begin end."
    )
}

fn codes(source: &str, code: DiagnosticCode) -> usize {
    check_errors(source)
        .iter()
        .filter(|error| error.code == code)
        .count()
}

#[test]
fn is_tests_bind_in_conditions_and_bodies() {
    check_ok(&program(
        "if S is Shape.Circle(const Radius) and Radius > 0 then
           const Area: integer := Radius * Radius;
         elsif S is Shape.Point then
           null;
         end if;
         while O is Some(const Value) and Value < 10 do
           break;
         end while;
         if R is Ok(Some(const Value)) and O is Some(const Other) and Value = Other then
           null;
         end if;
         if S is Shape.Circle(_) then null; end if;",
    ));
}

#[test]
fn bindings_are_not_visible_outside_the_guarded_body() {
    for body in [
        "if O is Some(const Value) then null; else const Copy: integer := Value; end if;",
        "if O is Some(const Value) then null; end if; const Copy: integer := Value;",
        "while O is Some(const Value) do break; end while; const Copy: integer := Value;",
    ] {
        assert!(codes(&program(body), SEMA_UNKNOWN_NAME) >= 1, "{body}");
    }
}

#[test]
fn is_outside_a_branch_condition_is_rejected() {
    for body in [
        "const Hit: boolean := O is Some(_);",
        "if O is Some(_) or true then null; end if;",
        "if not (O is Some(_)) then null; end if;",
        "if true and (O is Some(_)) then null; end if;",
        "repeat null; until O is None;",
        "case 1 of when 1 if O is Some(_): null; else null; end case;",
    ] {
        assert_eq!(codes(&program(body), SEMA_MISPLACED_IS_TEST), 1, "{body}");
    }
}

#[test]
fn is_patterns_follow_case_pattern_rules() {
    assert_eq!(
        codes(
            &program("if O is Some(Value) then null; end if;"),
            SEMA_IMPLICIT_PATTERN_BINDING
        ),
        1
    );
    assert_eq!(
        codes(
            &program("if O is Shape.Point then null; end if;"),
            SEMA_TYPE_MISMATCH
        ),
        1
    );
    assert_eq!(
        codes(
            &program("if O is Some(const V) and R is Ok(Some(const V)) then null; end if;"),
            SEMA_DUPLICATE_DECLARATION
        ),
        1
    );
    assert_eq!(
        codes(&program("if O is _ then null; end if;"), SEMA_TYPE_MISMATCH),
        1
    );
}

#[test]
fn is_variant_qualifiers_must_resolve_to_the_tested_type() {
    for body in [
        "if O is Some(Missing.Circle(_)) then null; end if;",
        "if S is Missing.Point then null; end if;",
    ] {
        let source = program(body).replace("O: option of integer", "O: option of Shape");
        assert_eq!(codes(&source, SEMA_UNKNOWN_NAME), 1, "{body}");
    }
    assert_eq!(
        codes(
            "program T;
             type Shape = enum Circle(Radius: integer); Point; end enum;
             type Other = enum Point; end enum;
             procedure P(O: option of Shape);
             begin if O is Some(Other.Point) then null; end if; end procedure;
             begin end.",
            SEMA_TYPE_MISMATCH
        ),
        1
    );
}

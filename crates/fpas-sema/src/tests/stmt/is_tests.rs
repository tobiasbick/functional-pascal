//! Nonbinding `Value is Pattern` Boolean expressions and rejected bindings.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/is-test.md`

use super::super::{check_errors, check_ok};
use fpas_diagnostics::DiagnosticCode;
use fpas_diagnostics::codes::{
    SEMA_IMPLICIT_PATTERN_BINDING, SEMA_IS_PATTERN_BINDING, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME,
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
fn nonbinding_is_is_boolean_in_all_expression_contexts() {
    check_ok(&program(
        "const Present: boolean := (O is Some(_));
         const Flags: array of boolean := [O is Some(_), not (O is Some(_)), (O is None) xor true];
         if (R is Ok(Some(_))) or (O is None) then null; end if;
         while (O is Some(_)) and false do break; end while;
         repeat null; until O is None;
         case 1 of when 1 if O is Some(_): null; else null; end case;",
    ));
    check_ok(
        "program T;
         function Has(Value: Option of integer): boolean;
         requires Value is Some(_);
         ensures (Present): Present;
         begin return Value is Some(_); end function;
         procedure Accept(Flag: boolean); ensures (Flag); begin null; end procedure;
         begin Accept(Has(Some(1))); end.",
    );
}

#[test]
fn is_never_accepts_bindings_in_any_expression_context() {
    for body in [
        "if O is Some(const Value) then null; end if;",
        "if false then null; elsif O is Some(const Value) then null; end if;",
        "while O is Some(const Value) do break; end while;",
        "repeat null; until O is Some(const Value);",
        "const Hit: boolean := (O is Some(const Value));",
        "const Hit: boolean := not (R is Ok(Some(const Value)));",
        "const Hit: boolean := (O is Some(const Value)) or true;",
        "const Hit: boolean := true xor (O is Some(const Value));",
        "case 1 of when 1 if O is Some(const Value): null; else null; end case;",
        "const Hit: boolean := if true then O is Some(const Value) else false end if;",
        "const Hit: boolean := S is Shape.Circle(const Radius);",
        "const Hit: boolean := O is const Value;",
    ] {
        let errors = check_errors(&program(body));
        assert_eq!(errors.len(), 1, "{body}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_IS_PATTERN_BINDING, "{body}");
        assert!(
            errors[0]
                .help
                .as_deref()
                .is_some_and(|hint| hint.contains("case arm"))
        );
    }
    for clause in ["requires", "ensures"] {
        let source = format!(
            "program T; function Has(O: Option of integer): boolean;
            {clause} O is Some(const Value); begin return true; end function; begin end."
        );
        assert_eq!(codes(&source, SEMA_IS_PATTERN_BINDING), 1, "{clause}");
    }
}

#[test]
fn rejected_is_bindings_never_introduce_names() {
    for body in [
        "if O is Some(const Value) then null; else const Copy: integer := Value; end if;",
        "if O is Some(const Value) then null; end if; const Copy: integer := Value;",
        "while O is Some(const Value) do break; end while; const Copy: integer := Value;",
    ] {
        assert_eq!(codes(&program(body), SEMA_IS_PATTERN_BINDING), 1, "{body}");
        assert_eq!(codes(&program(body), SEMA_UNKNOWN_NAME), 1, "{body}");
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
            SEMA_IS_PATTERN_BINDING
        ),
        2
    );
    for body in [
        "if O is _ then null; end if;",
        "const Hit: boolean := (O is _);",
        "const Hit: Option of boolean := Some(O is _);",
        "const Hits: array of boolean := [O is _];",
        "const Hit: boolean := (S is _);",
    ] {
        assert_eq!(codes(&program(body), SEMA_TYPE_MISMATCH), 1, "{body}");
    }
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

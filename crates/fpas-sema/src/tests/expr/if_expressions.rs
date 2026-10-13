//! Branch types, Boolean conditions, and constants of `if` expressions.
//!
//! Documentation: `docs/pascal/language/control-flow/if-then-else.md`

use fpas_diagnostics::codes::{
    SEMA_NON_BOOLEAN_CONDITION, SEMA_NON_CONSTANT_EXPRESSION, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME,
};

use super::{check_errors, check_ok};

const PRELUDE: &str = "program T;
  type UserId = distinct integer;
  function Load(): integer; begin return 1; end function;";

fn program(body: &str) -> String {
    format!("{PRELUDE}\nbegin\n  const N: integer := Load();\n{body}\nend.")
}

fn single_error(body: &str) -> crate::SemaError {
    let errors = check_errors(&program(body));
    assert_eq!(errors.len(), 1, "{body}: {errors:#?}");
    errors.into_iter().next().unwrap_or_else(|| unreachable!())
}

#[test]
fn branches_share_one_type_and_take_context_from_each_other() {
    check_ok(&program(
        "const Noun: string := if N = 1 then 'item' else 'items' end if;
         const Grade: string := if N > 90 then 'A' elsif N > 50 then 'B' else 'C' end if;
         const Picked: option of integer := if N = 1 then None else Some(1) end if;
         const Items: array of integer := if N = 1 then [] else [1, 2] end if;
         const Sum: integer := 1 + if N > 0 then 2 else 3 end if;
         const Id: UserId := if N = 1 then UserId(1) else UserId(2) end if;
         const Nested: integer := if N = 1 then if N > 0 then 1 else 2 end if else 3 end if;",
    ));
}

#[test]
fn branches_are_not_widened_or_converted() {
    let error = single_error("const R: real := if N = 1 then 1 else 2.5 end if;");
    assert_eq!(error.code, SEMA_TYPE_MISMATCH);
    assert!(
        error
            .message
            .contains("`if` expression branches have different types: `integer` and `real`"),
        "{error:#?}"
    );
    assert!(
        error
            .help
            .as_deref()
            .is_some_and(|help| help.contains("`IntToReal(...)`")),
        "{error:#?}"
    );
    let error = single_error("const Id: UserId := if N = 1 then UserId(1) else 2 end if;");
    assert!(
        error
            .help
            .as_deref()
            .is_some_and(|help| help.contains("`UserId(Value)`")),
        "{error:#?}"
    );
}

#[test]
fn the_shared_type_is_checked_against_the_expected_type() {
    let error = single_error("const S: string := if N = 1 then 1 else 2 end if;");
    assert_eq!(error.code, SEMA_TYPE_MISMATCH);
    assert!(
        error.message.contains("expected `string`, found `integer`"),
        "{error:#?}"
    );
}

#[test]
fn complementary_result_branches_preserve_every_payload_type() {
    for (ty, left, right) in [
        (
            "Result of (integer, string)",
            "Error('reason')",
            "Ok('wrong')",
        ),
        ("Result of (integer, string)", "Ok(1)", "Error(2)"),
        (
            "Option of Result of (integer, string)",
            "Some(Error('reason'))",
            "Some(Ok('wrong'))",
        ),
        (
            "array of Result of (integer, string)",
            "[Error('reason')]",
            "[Ok('wrong')]",
        ),
    ] {
        for (first, second) in [(left, right), (right, left)] {
            for value in [
                format!("if N = 1 then {first} else {second} end if"),
                format!("case N of when 1: {first}; else {second}; end case"),
            ] {
                let error = single_error(&format!("const Value: {ty} := {value};"));
                assert_eq!(error.code, SEMA_TYPE_MISMATCH, "{error:#?}");
                assert!(
                    error.expected.is_some() && error.found.is_some(),
                    "{error:#?}"
                );
            }
        }
    }
}

#[test]
fn complementary_result_evidence_rejects_a_conflicting_third_branch() {
    for value in [
        "if N = 1 then Error('reason') elsif N = 2 then Ok(1) else Ok('wrong') end if",
        "case N of when 1: Error('reason'); when 2: Ok(1); else Ok('wrong'); end case",
    ] {
        let error = single_error(&format!(
            "const Value: Result of (integer, string) := {value};"
        ));
        assert_eq!(error.code, SEMA_TYPE_MISMATCH, "{error:#?}");
    }
    check_ok(&program(
        "const A: Result of (integer, string) := if N = 1 then Error('reason') else Ok(1) end if;
         const B: Result of (integer, string) := case N of when 1: Ok(1); else Error('reason'); end case;
         const C: Option of Result of (integer, string) := if N = 1 then Some(Error('reason')) else Some(Ok(1)) end if;",
    ));
}

#[test]
fn conditions_are_boolean_and_case_bindings_stay_in_their_arm() {
    let error = single_error("const X: integer := if N then 1 else 2 end if;");
    assert_eq!(error.code, SEMA_NON_BOOLEAN_CONDITION);
    check_ok(&program(
        "const Found: option of integer := Some(N);
         const X: integer := if Found is Some(_) then N elsif N = 0 then 0 else -1 end if;",
    ));
    let error = single_error(
        "const Found: option of integer := Some(N);
         const X: integer := case Found of when Some(const V): V; when None: V; end case;",
    );
    assert_eq!(error.code, SEMA_UNKNOWN_NAME, "{error:#?}");
}

#[test]
fn constant_parts_make_a_compile_time_constant() {
    check_ok(
        "program T;
         const Debug: boolean := true;
         const Limit: integer := if Debug then 10 else 100 end if;
         begin
           case 10 of when Limit: null; else null; end case;
         end.",
    );
    let error = single_error(
        "const Computed: integer := if N = 1 then 10 else 100 end if;
         case 10 of when Computed: null; else null; end case;",
    );
    assert_eq!(error.code, SEMA_NON_CONSTANT_EXPRESSION, "{error:#?}");
}

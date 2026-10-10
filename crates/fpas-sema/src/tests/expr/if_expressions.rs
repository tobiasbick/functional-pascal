//! Branch types, conditions, `is` bindings, and constants of `if` expressions.
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
fn conditions_are_boolean_and_is_bindings_stay_in_their_branch() {
    let error = single_error("const X: integer := if N then 1 else 2 end if;");
    assert_eq!(error.code, SEMA_NON_BOOLEAN_CONDITION);
    check_ok(&program(
        "const Found: option of integer := Some(N);
         const X: integer := if Found is Some(const V) and V > 0 then V elsif N = 0 then 0 else -1 end if;",
    ));
    let error = single_error(
        "const Found: option of integer := Some(N);
         const X: integer := if Found is Some(const V) then V else V end if;",
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

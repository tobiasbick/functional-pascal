//! Arm types, coverage, bindings, guards, and constants of `case` expressions.
//!
//! Documentation: `docs/pascal/language/control-flow/case-of-intro.md`

use fpas_diagnostics::codes::{
    SEMA_CLOSED_ENUM_ELSE, SEMA_NON_CONSTANT_EXPRESSION, SEMA_NON_EXHAUSTIVE_CASE,
    SEMA_TYPE_MISMATCH,
};

use super::{check_errors, check_ok};

const PRELUDE: &str = "program T;
  type Shape = enum Circle(Radius: real); Square(Side: real); end enum;
  type Tint = enum Red; Green; end enum;
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
fn complete_cases_produce_one_shared_type() {
    check_ok(&program(
        "const S: Shape := Shape.Circle(1.0);
         const Area: real := case S of
           when Shape.Circle(const R): 3.0 * R * R;
           when Shape.Square(const Side): Side * Side;
         end case;
         const Name: string := case Tint.Red of when Tint.Red: 'r'; when Tint.Green: 'g'; end case;
         const Size: string := case N of when 1..9: 'small'; when const Big if Big > 100: 'huge'; else 'mid'; end case;
         const Truth: string := case N > 1 of when true: 'yes'; when false: 'no'; end case;
         const Found: integer := case Some(N) of when Some(const V) if V > 3: V; when Some(const V): -V; when None: 0; end case;
         const Picked: option of integer := case N of when 1: None; else Some(N); end case;
         const Id: string := case UserId(N) of when UserId(1): 'admin'; else 'user'; end case;
         const Sum: integer := 1 + case N of when 1: 2; else 3; end case;",
    ));
}

#[test]
fn incomplete_cases_are_rejected() {
    let error = single_error("const S: string := case N of when 1: 'one'; end case;");
    assert_eq!(error.code, SEMA_NON_EXHAUSTIVE_CASE);
    assert!(
        error.message.contains("over `integer` needs an `else` arm"),
        "{error:#?}"
    );
    let error = single_error("const S: string := case Tint.Red of when Tint.Red: 'r'; end case;");
    assert_eq!(error.code, SEMA_NON_EXHAUSTIVE_CASE);
    assert!(
        error
            .message
            .contains("Non-exhaustive case expression: missing Tint.Green"),
        "{error:#?}"
    );
    let error = single_error("const B: boolean := case N > 1 of when true: true; end case;");
    assert!(error.message.contains("missing false"), "{error:#?}");
    let error = single_error(
        "const S: string := case Tint.Red of when Tint.Red: 'r'; when Tint.Green if N > 0: 'g'; end case;",
    );
    assert_eq!(error.code, SEMA_NON_EXHAUSTIVE_CASE, "{error:#?}");
}

#[test]
fn closed_enums_reject_else_like_the_statement() {
    let error =
        single_error("const S: string := case Tint.Red of when Tint.Red: 'r'; else 'x'; end case;");
    assert_eq!(error.code, SEMA_CLOSED_ENUM_ELSE);
}

#[test]
fn arm_values_share_one_type_and_match_the_expected_type() {
    let error = single_error("const S: string := case N of when 1: 'one'; else 2; end case;");
    assert_eq!(error.code, SEMA_TYPE_MISMATCH);
    assert!(
        error
            .message
            .contains("`case` expression arms have different types: `string` and `integer`"),
        "{error:#?}"
    );
    let error = single_error("const S: string := case N of when 1: 1; else 2; end case;");
    assert!(
        error.message.contains("expected `string`, found `integer`"),
        "{error:#?}"
    );
}

#[test]
fn constant_selectors_labels_and_values_make_a_compile_time_constant() {
    check_ok(
        "program T;
         const Mode: integer := 2;
         const Limit: integer := case Mode of when 1: 10; when 2..5: 20; else 30; end case;
         begin
           case 20 of when Limit: null; else null; end case;
         end.",
    );
    let error = single_error(
        "const Computed: integer := case N of when 1: 10; else 20; end case;
         case 10 of when Computed: null; else null; end case;",
    );
    assert_eq!(error.code, SEMA_NON_CONSTANT_EXPRESSION, "{error:#?}");
}

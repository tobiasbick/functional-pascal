//! Case coverage is verified at the language boundary in both syntactic positions.

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{
    SEMA_CASE_ELSE_FORBIDDEN, SEMA_NON_EXHAUSTIVE_CASE, SEMA_UNREACHABLE_CASE_LABEL,
};

fn source(prelude: &str, selector: &str, arms: &str, expression: bool) -> String {
    let arms = arms.replace("$body", if expression { "7" } else { "null" });
    let statement = if expression {
        format!("discard case {selector} of {arms} end case;")
    } else {
        format!("case {selector} of {arms} end case;")
    };
    format!("program T; {prelude} begin {statement} end.")
}

#[test]
fn statements_and_expressions_share_the_approved_coverage_matrix() {
    let cases = [
        (
            "",
            "true",
            "when true: $body;",
            Some(SEMA_NON_EXHAUSTIVE_CASE),
        ),
        (
            "",
            "true",
            "when true: $body; else $body;",
            Some(SEMA_CASE_ELSE_FORBIDDEN),
        ),
        ("", "true", "when true: $body; when false: $body;", None),
        (
            "",
            "true",
            "when true: $body; when false: $body; else $body;",
            Some(SEMA_CASE_ELSE_FORBIDDEN),
        ),
        (
            "",
            "true",
            "when true if true: $body; when false: $body;",
            Some(SEMA_NON_EXHAUSTIVE_CASE),
        ),
        (
            "type Digit = 1..3;",
            "Digit(2)",
            "when 1, 2, 3: $body;",
            None,
        ),
        (
            "type Digit = 1..3;",
            "Digit(2)",
            "when 1..2: $body; when 3: $body;",
            None,
        ),
        (
            "type Digit = 1..3;",
            "Digit(2)",
            "when 1, 3: $body;",
            Some(SEMA_NON_EXHAUSTIVE_CASE),
        ),
        (
            "type Digit = 1..3;",
            "Digit(2)",
            "when 1..3: $body; else $body;",
            Some(SEMA_CASE_ELSE_FORBIDDEN),
        ),
        (
            "type Digit = 1..3; type Alias = Digit;",
            "Alias(2)",
            "when 0..2: $body; when 3: $body;",
            None,
        ),
        ("", "2", "when 2: $body;", Some(SEMA_NON_EXHAUSTIVE_CASE)),
        ("", "2", "when 1..3: $body; else $body;", None),
        (
            "",
            "'a'",
            "when 'a': $body;",
            Some(SEMA_NON_EXHAUSTIVE_CASE),
        ),
        ("", "'a'", "when 'a': $body; else $body;", None),
        (
            "type Flag = distinct boolean;",
            "Flag(true)",
            "when Flag(true): $body; when Flag(false): $body;",
            None,
        ),
        (
            "type Flag = distinct boolean;",
            "Flag(true)",
            "when Flag(true): $body; else $body;",
            Some(SEMA_CASE_ELSE_FORBIDDEN),
        ),
        (
            "type Flag = distinct boolean;",
            "Flag(true)",
            "when Flag(true): $body;",
            Some(SEMA_NON_EXHAUSTIVE_CASE),
        ),
        (
            "type Id = distinct integer;",
            "Id(1)",
            "when Id(1): $body;",
            Some(SEMA_NON_EXHAUSTIVE_CASE),
        ),
        (
            "type Id = distinct integer;",
            "Id(1)",
            "when Id(1): $body; else $body;",
            None,
        ),
        (
            "type Text = distinct string;",
            "Text('a')",
            "when Text('a'): $body; else $body;",
            None,
        ),
        (
            "type Mode = enum First; Second; end enum;",
            "Mode.First",
            "when Mode.First: $body; when Mode.Second: $body;",
            None,
        ),
        (
            "type Mode = enum First; Second; Third; end enum;",
            "Mode.First",
            "when Mode.First: $body; when Mode.Second: $body;",
            Some(SEMA_NON_EXHAUSTIVE_CASE),
        ),
        (
            "type Digit = 1..4;",
            "Digit(2)",
            "when 1..3: $body;",
            Some(SEMA_NON_EXHAUSTIVE_CASE),
        ),
    ];
    for (prelude, selector, arms, expected) in cases {
        for expression in [false, true] {
            let source = source(prelude, selector, arms, expression);
            if let Some(code) = expected {
                let errors = check_errors(&source);
                assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
                assert_eq!(errors[0].code, code, "{source}: {errors:#?}");
                if selector == "Flag(true)" {
                    assert!(errors[0].message.contains("Flag(false)"), "{errors:#?}");
                }
            } else {
                check_ok(&source);
            }
        }
    }
}

#[test]
fn scalar_and_pattern_reachability_share_guard_and_overlap_rules() {
    let cases = [
        ("", "when 1, 1: $body; else $body;", true),
        ("", "when 1, 1 if true: $body; else $body;", true),
        ("", "when 1, 1..1 if true: $body; else $body;", true),
        ("", "when 1..1, 1 if true: $body; else $body;", true),
        (
            "const Limit: integer := 1;",
            "when Limit: $body; when 1 + 0: $body; else $body;",
            true,
        ),
        (
            "",
            "when 1: $body; when 1 if true: $body; else $body;",
            true,
        ),
        (
            "",
            "when 1 if true: $body; when 1: $body; else $body;",
            false,
        ),
        ("", "when 1..5: $body; when 3..8: $body; else $body;", false),
        ("", "when 1..2, 2..3 if false: $body; else $body;", false),
        ("", "when 1..3, 2 if false: $body; else $body;", false),
        (
            "",
            "when 1..2: $body; when 3..5: $body; when 2..4: $body; else $body;",
            true,
        ),
        ("", "when 3..1: $body; else $body;", true),
        (
            "",
            "when 1..5: $body; when const N if N > 10: $body; else $body;",
            false,
        ),
    ];
    for (prelude, arms, rejected) in cases {
        for expression in [false, true] {
            let source = source(prelude, "2", arms, expression);
            if rejected {
                let errors = check_errors(&source);
                assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
                assert_eq!(
                    errors[0].code, SEMA_UNREACHABLE_CASE_LABEL,
                    "{source}: {errors:#?}"
                );
            } else {
                check_ok(&source);
            }
        }
    }
    for expression in [false, true] {
        for (prelude, selector, arms) in [
            (
                "",
                "true",
                "when true, true..true if true: $body; when true: $body; when false: $body;",
            ),
            (
                "type Small = 1..3;",
                "Small(2)",
                "when 0..3, 1..4 if true: $body; when 1..3: $body;",
            ),
        ] {
            let source = source(prelude, selector, arms, expression);
            let errors = check_errors(&source);
            assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
            assert_eq!(errors[0].code, SEMA_UNREACHABLE_CASE_LABEL);
        }
        let partial = source(
            "",
            "'m'",
            "when 'a'..'z': $body; when 'm'..'zz': $body; else $body;",
            expression,
        );
        check_ok(&partial);
        let covered = source(
            "",
            "'m'",
            "when 'a'..'z': $body; when 'm': $body; else $body;",
            expression,
        );
        let errors = check_errors(&covered);
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert_eq!(errors[0].code, SEMA_UNREACHABLE_CASE_LABEL);
    }
}

#[test]
fn nested_finite_payloads_preserve_correlations_and_precise_missing_values() {
    let prelude = "type Digit = 1..2; type Pair = enum Item(Left: Digit; Right: Digit); end enum;";
    for expression in [false, true] {
        let incomplete = source(
            prelude,
            "Pair.Item(Digit(1), Digit(1))",
            "when Pair.Item(1, _): $body; when Pair.Item(2, 1): $body;",
            expression,
        );
        let errors = check_errors(&incomplete);
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert_eq!(errors[0].code, SEMA_NON_EXHAUSTIVE_CASE);
        assert!(errors[0].message.contains("Pair.Item(2, 2)"), "{errors:#?}");
        check_ok(&source(
            prelude,
            "Pair.Item(Digit(1), Digit(1))",
            "when Pair.Item(1, _): $body; when Pair.Item(2, 1): $body; when Pair.Item(2, 2): $body;",
            expression,
        ));
        let prelude = "type Digit = 1..2; const Found: option of Digit := Some(Digit(1));";
        check_ok(&source(
            prelude,
            "Found",
            "when Some(1): $body; when Some(2): $body; when None: $body;",
            expression,
        ));
        let errors = check_errors(&source(
            prelude,
            "Found",
            "when Some(1): $body; when None: $body;",
            expression,
        ));
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert!(errors[0].message.contains("Some(2)"), "{errors:#?}");
    }
}

#[test]
fn machine_wide_subrange_coverage_uses_intervals_and_open_integers_still_need_else() {
    let prelude = "const Minimum: integer := -9223372036854775807 - 1;
        const Maximum: integer := 9223372036854775807;
        type Full = Minimum..Maximum;";
    for expression in [false, true] {
        check_ok(&source(
            prelude,
            "Full(0)",
            "when Minimum..-1: $body; when 0..Maximum: $body;",
            expression,
        ));
        let errors = check_errors(&source(
            prelude,
            "Full(0)",
            "when Minimum..-1: $body; when 1..Maximum: $body;",
            expression,
        ));
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert_eq!(errors[0].code, SEMA_NON_EXHAUSTIVE_CASE);
        assert!(errors[0].message.contains("missing 0"), "{errors:#?}");
        let errors = check_errors(&source(
            prelude,
            "0",
            "when Minimum..Maximum: $body;",
            expression,
        ));
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert_eq!(errors[0].code, SEMA_NON_EXHAUSTIVE_CASE);
    }
}

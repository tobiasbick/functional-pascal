//! Type operands, value operands, and constant classification of subrange membership.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_NON_CONSTANT_EXPRESSION, SEMA_TYPE_MISMATCH};

const TYPES: &str = "program T; type Percent = 0..100; type Alias = Percent;
    type Small = -10..10; type Id = distinct integer; type State = enum Ready; Busy; end enum;";

#[test]
fn membership_accepts_integer_and_subrange_values_and_type_aliases() {
    check_ok(&format!(
        "{TYPES} begin const P := Percent(50);
        const Same: boolean := P in Alias; const Different: boolean := P in Small;
        const Combined: boolean := 50 in Percent and 100 in Alias;
        const Parenthesized: boolean := 0 in (Percent);
        const Negated: boolean := not 101 in Percent;
        const Existing: boolean := P in [50, 100];
        const Keys: boolean := P in [50: 'fifty'];
        const Text: boolean := 'a' in 'abc'; end."
    ));
}

#[test]
fn membership_rejects_non_integer_values_and_type_names_on_the_left() {
    for left in [
        "1.0",
        "true",
        "'1'",
        "State.Ready",
        "Id(1)",
        "[1]",
        "Percent",
        "integer",
    ] {
        let errors = check_errors(&format!(
            "{TYPES} begin const Bad: boolean := {left} in Percent; end."
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
                && error.message.contains("integer or subrange value")
                && error
                    .help
                    .as_deref()
                    .is_some_and(|help| help.contains("Input in Percent"))),
            "{left}: {errors:?}"
        );
    }
}

#[test]
fn membership_requires_a_subrange_type_rather_than_a_subrange_value() {
    for right in [
        "P",
        "Percent(50)",
        "Read()",
        "if true then P else P end if",
        "integer",
        "Numbers",
        "Text",
    ] {
        let errors = check_errors(&format!(
            "{TYPES} type Numbers = array of integer; type Text = string;
            function Read(): Percent; begin return Percent(50); end function;
            begin const P := Percent(50); const Bad: boolean := 50 in {right}; end."
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
                && (error.message.contains("subrange type")
                    || error.message.contains("integer subrange"))),
            "{right}: {errors:?}"
        );
    }
    check_ok(&format!(
        "{TYPES} begin const Percent := [1, 2]; const Found: boolean := 1 in Percent; end."
    ));
    let errors = check_errors(&format!(
        "{TYPES} begin const Percent := Alias(50); const Bad: boolean := 50 in Percent; end."
    ));
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "{errors:?}"
    );
}

#[test]
fn static_membership_is_folded_but_runtime_dependencies_remain_computed() {
    let source = format!("{TYPES} const Inside: boolean := 50 in Percent;
        const Outside: boolean := 101 in Percent;
        type Selected = 0..if Inside and not Outside then 1 else 2 end if;
        begin const P := Selected(1); case true of when Inside: null; when Outside: null; end case; end.");
    check_ok(&source);
    let errors = check_errors(&source.replace("Selected(1)", "Selected(2)"));
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
            && error.message.contains("outside")
            && error.message.contains("0..1")),
        "{errors:?}"
    );
    let errors = check_errors(&format!("{TYPES} function Read(): integer; begin return 50; end function;
        begin const Found: boolean := Read() in Percent; case true of when Found: null; end case; end."));
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_NON_CONSTANT_EXPRESSION
                && error.message.contains("Found")),
        "{errors:?}"
    );
}

#[test]
fn membership_does_not_implicitly_narrow_an_integer_value() {
    let errors = check_errors(&format!(
        "{TYPES} begin var Input: integer := 50;
        if Input in Percent then const P: Percent := Input; end if; end."
    ));
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
            && error
                .help
                .as_deref()
                .is_some_and(|help| help.contains("checked conversion"))),
        "{errors:?}"
    );
}

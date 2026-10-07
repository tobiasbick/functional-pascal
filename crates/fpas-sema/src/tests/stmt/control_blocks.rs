//! Lexical visibility of AP13.4 branch and loop statement lists.

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_UNKNOWN_NAME;

#[test]
fn each_branch_and_loop_body_has_its_own_scope() {
    for body in [
        "if true then const Hidden: integer := 1; end if;",
        "if false then null; elsif true then const Hidden: integer := 1; end if;",
        "if false then null; else const Hidden: integer := 1; end if;",
        "while false do const Hidden: integer := 1; end while;",
        "for I: integer := 1 to 2 do const Hidden: integer := I; end for;",
        "for I: integer in [1, 2] do const Hidden: integer := I; end for;",
        "repeat const Hidden: integer := 1; until true;",
    ] {
        let errors = check_errors(&format!(
            "program T; begin {body} const Outside: integer := Hidden; end."
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_UNKNOWN_NAME),
            "{body}: {errors:#?}"
        );
    }
}

#[test]
fn branch_local_names_do_not_enter_later_conditions_or_other_branches() {
    for body in [
        "if true then const Hidden: boolean := true; elsif Hidden then null; end if;",
        "if true then const Hidden: integer := 1; else const Copy: integer := Hidden; end if;",
        "while Hidden do const Hidden: boolean := true; end while;",
        "repeat const Hidden: boolean := true; until Hidden;",
    ] {
        let errors = check_errors(&format!("program T; begin {body} end."));
        assert!(
            errors.iter().any(|error| error.code == SEMA_UNKNOWN_NAME),
            "{body}: {errors:#?}"
        );
    }
}

#[test]
fn branch_lists_support_shadowing_and_multiple_declarations() {
    check_ok(
        "program T; begin const Value: integer := 7;
        if true then const Value: string := 'branch'; const Copy: string := Value;
        elsif false then const Value: boolean := true;
        else const Value: real := 1.0; end if;
        const Original: integer := Value;
        while false do const Value: string := 'loop'; const Copy: string := Value; end while;
        const StillOriginal: integer := Value; end.",
    );
}

#[test]
fn explicit_compound_blocks_retain_an_additional_scope() {
    let errors = check_errors(
        "program T; begin if true then
        begin const Hidden: integer := 1; end;
        const Outside: integer := Hidden; end if; end.",
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_UNKNOWN_NAME),
        "{errors:#?}"
    );
}

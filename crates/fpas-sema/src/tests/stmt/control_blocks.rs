//! Lexical visibility of AP13.4 branch and loop statement lists.

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_UNKNOWN_NAME;

#[test]
fn each_branch_and_loop_body_has_its_own_scope() {
    for body in [
        "if true then var Hidden: integer := 1; end if;",
        "if false then null; elsif true then var Hidden: integer := 1; end if;",
        "if false then null; else var Hidden: integer := 1; end if;",
        "while false do var Hidden: integer := 1; end while;",
        "for I: integer := 1 to 2 do var Hidden: integer := I; end for;",
        "for I: integer in [1, 2] do var Hidden: integer := I; end for;",
        "repeat var Hidden: integer := 1; until true;",
    ] {
        let errors = check_errors(&format!(
            "program T; begin {body} var Outside: integer := Hidden; end."
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
        "if true then var Hidden: boolean := true; elsif Hidden then null; end if;",
        "if true then var Hidden: integer := 1; else var Copy: integer := Hidden; end if;",
        "while Hidden do var Hidden: boolean := true; end while;",
        "repeat var Hidden: boolean := true; until Hidden;",
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
        "program T; begin var Value: integer := 7;
        if true then var Value: string := 'branch'; var Copy: string := Value;
        elsif false then var Value: boolean := true;
        else var Value: real := 1.0; end if;
        var Original: integer := Value;
        while false do var Value: string := 'loop'; var Copy: string := Value; end while;
        var StillOriginal: integer := Value; end.",
    );
}

#[test]
fn explicit_compound_blocks_retain_an_additional_scope() {
    let errors = check_errors(
        "program T; begin if true then
        begin var Hidden: integer := 1; end;
        var Outside: integer := Hidden; end if; end.",
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_UNKNOWN_NAME),
        "{errors:#?}"
    );
}

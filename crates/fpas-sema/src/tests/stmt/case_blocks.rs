//! Visibility of AP13.5 case-arm locals and existing pattern bindings.

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_UNKNOWN_NAME;

#[test]
fn all_arm_locals_are_hidden_after_the_case() {
    for body in [
        "case 1 of when 1: const Hidden: integer := 1; end case;",
        "case 1 of when 0: null; else const Hidden: integer := 1; end case;",
        "case Some(1) of when Some(const V): const Hidden: integer := V; when None: null; end case;",
    ] {
        let errors = check_errors(&format!(
            "program T; begin {body} const Outside: integer := Hidden; end."
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_UNKNOWN_NAME),
            "{errors:#?}"
        );
    }
}

#[test]
fn locals_do_not_escape_into_later_guards_or_the_catch_all() {
    for body in [
        "case 1 of when 1: const Hidden: boolean := true; when 2 if Hidden: null; end case;",
        "case 1 of when 1: const Hidden: integer := 1; else const Copy: integer := Hidden; end case;",
        "case 1 of when 1: begin const Hidden: integer := 1; end; const Copy: integer := Hidden; end case;",
    ] {
        let errors = check_errors(&format!("program T; begin {body} end."));
        assert!(
            errors.iter().any(|error| error.code == SEMA_UNKNOWN_NAME),
            "{errors:#?}"
        );
    }
}

#[test]
fn arms_support_shadowing_and_existing_pattern_guard_bindings() {
    check_ok(
        "program T; begin const Value: integer := 7; case Value of when 1: const Value: string := 'arm'; const Copy: string := Value; else const Value: boolean := true; end case; const Original: integer := Value; case Some(1) of when Some(const V) if V > 0: const Copy: integer := V; when Some(const V): null; when None: null; end case; end.",
    );
    check_ok(
        "program T; begin case 1 of when const N if N > 0: const Copy: integer := N; else null; end case; end.",
    );
}

#[test]
fn catch_all_on_closed_types_remains_accepted() {
    check_ok(
        "program T; type Light = enum Red; Green; end enum; begin case Light.Red of when Light.Red: null; else null; end case; case Some(1) of when None: null; else null; end case; end.",
    );
}

//! Closed-enum catch-alls, extension checks, and scalar case boundaries.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/exhaustiveness.md`

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{
    SEMA_CLOSED_ENUM_ELSE, SEMA_NON_EXHAUSTIVE_CASE, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME,
};

fn closed_else(source: &str) -> crate::SemaError {
    let errors = check_errors(source);
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_CLOSED_ENUM_ELSE, "{errors:#?}");
    errors.into_iter().next().expect("closed-enum diagnostic")
}

#[test]
fn catch_alls_name_missing_variants_of_closed_types_and_aliases() {
    for (source, missing) in [
        (
            "program T; type Shape = enum Circle(Radius: integer); Point; end enum;
             begin case Shape.Point of when Shape.Circle(_): null; else null; end case; end.",
            "Shape.Point",
        ),
        (
            "program T; begin case Some(1) of when Some(_): null; else null; end case; end.",
            "None",
        ),
        (
            "program T; const R: result of integer, string := Ok(1);
             begin case R of when Ok(_): null; else null; end case; end.",
            "Error(_)",
        ),
        (
            "program T; type Color = enum Red; Green; end enum; type Alias = Color;
             const Value: Alias := Color.Red;
             begin case Value of when Alias.Red: null; else null; end case; end.",
            "Color.Green",
        ),
        (
            "program T; type Maybe = option of integer; const Value: Maybe := Some(1);
             begin case Value of when None: null; else null; end case; end.",
            "Some(_)",
        ),
        (
            "program T; type Answer = result of integer, string; const Value: Answer := Ok(1);
             begin case Value of when Error(_): null; else null; end case; end.",
            "Ok(_)",
        ),
    ] {
        let error = closed_else(source);
        assert!(error.message.contains(missing), "{error:#?}");
        let hint = error.help.expect("replacement hint");
        assert!(
            hint.contains("`when`") && hint.contains("`null;`"),
            "{hint}"
        );
        assert!(hint.contains("is Pattern"), "{hint}");
    }
}

#[test]
fn redundant_catch_alls_only_suggest_removing_the_branch() {
    for source in [
        "program T; type Color = enum Red; Green; end enum;
         begin case Color.Red of when Color.Red, Color.Green: null; else null; end case; end.",
        "program T; type Shape = enum Circle(Radius: integer); Point; end enum;
         begin case Shape.Point of when Shape.Circle(_), Shape.Point: null; else null; end case; end.",
        "program T; begin case Some(1) of when Some(_), None: null; else null; end case; end.",
        "program T; const R: result of integer, string := Ok(1);
         begin case R of when Ok(_), Error(_): null; else null; end case; end.",
    ] {
        let error = closed_else(source);
        assert!(!error.message.contains("missing"), "{error:#?}");
        let hint = error.help.expect("redundant-else hint");
        assert!(hint.contains("already covers every variant"), "{hint}");
        assert!(hint.contains("Remove the redundant `else`"), "{hint}");
        assert!(!hint.contains("is Pattern"), "{hint}");
    }
}

#[test]
fn guarded_and_partial_payload_arms_leave_missing_patterns() {
    for (source, missing) in [
        (
            "program T; type Shape = enum Circle(Radius: integer); Point; end enum;
             begin case Shape.Point of when Shape.Circle(const R) if R > 0: null;
             when Shape.Point: null; else null; end case; end.",
            "Shape.Circle(_)",
        ),
        (
            "program T; const R: result of option of integer, string := Ok(None);
             begin case R of when Ok(Some(_)): null; when Error(_): null; else null; end case; end.",
            "Ok(None)",
        ),
        (
            "program T; type Color = enum Red; Green; end enum;
             begin case Color.Red of when const C if true: null; else null; end case; end.",
            "Color.Green",
        ),
    ] {
        let error = closed_else(source);
        assert!(error.message.contains(missing), "{error:#?}");
    }
}

#[test]
fn a_payload_wildcard_cannot_cover_another_variant() {
    let errors = check_errors(
        "program T; type Shape = enum Circle(Radius: integer); Point; end enum;
         begin case Shape.Point of when Shape.Circle(_): null; end case; end.",
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_NON_EXHAUSTIVE_CASE);
    assert!(errors[0].message.contains("Shape.Point"));
    assert!(!errors[0].help.as_deref().expect("hint").contains("else"));
}

#[test]
fn enum_extension_reports_every_incomplete_case_and_accepts_complete_cases() {
    let program = |extended: bool| {
        let extra_member = if extended { "Blue;" } else { "" };
        let extra_label = if extended { ", Color.Blue" } else { "" };
        format!(
            "program T; type Color = enum Red; Green; {extra_member} end enum;
             procedure Inspect(Value: Color); begin
               case Value of when Color.Red, Color.Green: null; end case;
               case Value of when Color.Red if true: null;
                 when Color.Red, Color.Green: null; end case;
               case Value of when Color.Red, Color.Green{extra_label}: null; end case;
             end procedure; begin end."
        )
    };
    check_ok(&program(false));
    let errors = check_errors(&program(true));
    assert_eq!(errors.len(), 2, "{errors:#?}");
    for error in &errors {
        assert_eq!(error.code, SEMA_NON_EXHAUSTIVE_CASE);
        assert!(error.message.contains("Color.Blue"), "{error:#?}");
    }
    assert_ne!(errors[0].span, errors[1].span);
}

#[test]
fn scalar_catch_alls_and_partial_boolean_cases_remain_valid() {
    check_ok(
        "program T; type Flag = boolean; const Value: Flag := true; begin
           case 42 of when 1: null; else null; end case;
           case 'unknown' of when 'known': null; else null; end case;
           case Value of when true: null; else null; end case;
           case true of when true, false: null; else null; end case;
           case false of when true: null; end case;
         end.",
    );
}

#[test]
fn label_errors_do_not_produce_a_misleading_redundant_else_hint() {
    let errors = check_errors(
        "program T; begin case Some(1) of when Some('wrong'): null; else null; end case; end.",
    );
    assert!(errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH));
    let error = errors
        .iter()
        .find(|error| error.code == SEMA_CLOSED_ENUM_ELSE)
        .expect("catch-all rejection even after an invalid label");
    assert!(!error.help.as_deref().expect("hint").contains("redundant"));
}

#[test]
fn an_unknown_scrutinee_does_not_produce_a_closed_enum_error() {
    let errors =
        check_errors("program T; begin case Missing of when 0: null; else null; end case; end.");
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_UNKNOWN_NAME);
}

#[test]
fn rejected_else_bodies_are_checked_without_leaking_their_locals() {
    let errors = check_errors(
        "program T; begin case Some(1) of when Some(_), None: null;
         else const Hidden: integer := 1; end case;
         const Outside: integer := Hidden; end.",
    );
    assert_eq!(errors.len(), 2, "{errors:#?}");
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_CLOSED_ENUM_ELSE)
    );
    assert!(errors.iter().any(|error| error.code == SEMA_UNKNOWN_NAME));
}

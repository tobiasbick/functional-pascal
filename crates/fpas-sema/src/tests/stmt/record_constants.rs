//! Static record projections in finite coverage and duplicate-pattern checks.
//! See `docs/pascal/language/pattern-matching/exhaustiveness.md`.

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{
    SEMA_NON_CONSTANT_EXPRESSION, SEMA_NON_EXHAUSTIVE_CASE, SEMA_UNREACHABLE_CASE_LABEL,
};

const DECLARATIONS: &str = "
type Color = enum Red; Green; end enum;
type Flags = record Enabled: boolean := true; Color: Color := Color.Red; end record;
type Settings = record Flags: Flags; Values: array of integer; end record;
const Original: Flags := Flags( );
const Copy: Flags := Original;
const Updated: Flags := Original with Enabled := false; Color := Color.Green; end with;
const Config: Settings := Settings(Flags := Copy, Values := [1, 2]);
const Enabled: boolean := Config.Flags.Enabled;
const Transitive: boolean := Enabled;
const Selected: Color := Updated.Color;";

fn program(body: &str) -> String {
    format!("program T; {DECLARATIONS} begin {body} end.")
}

#[test]
fn direct_nested_default_updated_and_transitive_boolean_fields_complete_coverage() {
    for (label, other) in [
        ("Original.Enabled", "false"),
        ("Copy.Enabled", "false"),
        ("Config.Flags.Enabled", "false"),
        ("cOnFiG.fLaGs.eNaBlEd", "false"),
        ("(Config.Flags.Enabled)", "false"),
        ("Transitive", "false"),
        ("Updated.Enabled", "true"),
    ] {
        check_ok(&program(&format!(
            "const Candidate: Option of boolean := Some(true);
            case Candidate of when Some({label}): null; when Some({other}): null;
            when None: null; end case;"
        )));
    }
}

#[test]
fn simple_enum_fields_complete_nested_result_option_coverage() {
    for label in ["Updated.Color", "Selected"] {
        check_ok(&program(&format!(
            "const Candidate: result of (Option of Color, string) := Ok(Some(Color.Red));
            case Candidate of when Ok(Some(Original.Color)): null;
            when Ok(Some({label})): null; when Ok(None): null; when Error(_): null; end case;"
        )));
    }
}

#[test]
fn record_field_aliases_and_literals_report_duplicate_patterns() {
    for (ty, value, first, duplicate, rest) in [
        ("boolean", "true", "Original.Enabled", "true", "false"),
        (
            "boolean",
            "true",
            "Config.Flags.Enabled",
            "Transitive",
            "false",
        ),
        (
            "Color",
            "Color.Red",
            "Original.Color",
            "Color.Red",
            "Color.Green",
        ),
        (
            "Color",
            "Color.Green",
            "Updated.Color",
            "Selected",
            "Color.Red",
        ),
    ] {
        let errors = check_errors(&program(&format!(
            "const Candidate: Option of {ty} := Some({value}); case Candidate of
            when Some({first}): null; when Some({duplicate}): null;
            when Some({rest}): null; when None: null; end case;"
        )));
        assert_eq!(
            errors
                .iter()
                .filter(|error| error.code == SEMA_UNREACHABLE_CASE_LABEL)
                .count(),
            1,
            "{errors:#?}"
        );
        assert!(
            !errors
                .iter()
                .any(|error| error.code == SEMA_NON_EXHAUSTIVE_CASE),
            "{errors:#?}"
        );
    }
}

#[test]
fn known_fields_do_not_cover_the_opposite_boolean_value() {
    let errors = check_errors(&program(
        "const Candidate: Option of boolean := Some(false); case Candidate of
        when Some(Original.Enabled): null; when None: null; end case;",
    ));
    let missing = errors
        .iter()
        .find(|error| error.code == SEMA_NON_EXHAUSTIVE_CASE)
        .expect("incomplete coverage");
    assert!(missing.message.contains("Some(false)"), "{missing:?}");
}

#[test]
fn computed_and_mutable_records_remain_invalid_constant_patterns() {
    for binding in [
        "var Config: Flags := Flags(Enabled := true);",
        "const Config: Flags := Flags(Enabled := ReadFlag());",
        "const Config: Flags := Flags(Enabled := true) with Enabled := ReadFlag(); end with;",
    ] {
        let errors = check_errors(&format!(
            "program T; type Flags = record Enabled: boolean; end record;
            function ReadFlag(): boolean; begin return true; end function;
            begin {binding} const Candidate: Option of boolean := Some(true);
            case Candidate of when Some(Config.Enabled): null; when Some(_): null;
            when None: null; end case; end."
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_NON_CONSTANT_EXPRESSION),
            "{errors:#?}"
        );
    }
}

#[test]
fn defaults_retain_values_from_their_declaration_scope() {
    check_ok(
        "program T; const Enabled: boolean := true;
        type Flags = record Enabled: boolean := Enabled; end record;
        begin const Enabled: boolean := false; const Config: Flags := Flags( );
        const Candidate: Option of boolean := Some(true);
        case Candidate of when Some(Config.Enabled): null; when Some(false): null;
        when None: null; end case; end.",
    );
}

#[test]
fn nested_routines_see_hoisted_record_constants_and_scalar_projections() {
    check_ok(
        "program T; type Flags = record Enabled: boolean; end record;
        procedure Outer();
        procedure Inner(); begin const Candidate: Option of boolean := Some(true);
        case Candidate of when Some(Enabled): null; when Some(false): null;
        when None: null; end case; end procedure;
        begin const Config: Flags := Flags(Enabled := true);
        const Enabled: boolean := Config.Enabled; Inner(); end procedure;
        begin Outer(); end.",
    );
}

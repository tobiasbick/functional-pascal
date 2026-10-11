//! Scalar conversions require values, including across aliases and unit qualification.

use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_parser::{CompilationUnit, parse, parse_compilation_unit};

const TYPES: &str = "
type Small = 0..10; type SmallAlias = Small;
type UserId = distinct integer; type IdAlias = UserId;";

fn analyze(body: &str) -> Vec<crate::SemaError> {
    let (parsed, errors) = parse_compilation_unit(
        "unit Domain; public type Small = 0..10; public type UserId = distinct integer;
         public type RangeAlias = Small; public type IdAlias = UserId;
         public const Number: integer := 1; public const Id: UserId := UserId(1); end unit;",
    );
    assert!(errors.is_empty(), "{errors:#?}");
    let CompilationUnit::Unit(unit) = parsed else {
        panic!("expected unit");
    };
    let unit = crate::analyze_unit(&unit, &[]).expect("unit analysis");
    assert!(
        unit.metadata.errors.is_empty(),
        "{:#?}",
        unit.metadata.errors
    );
    let interface = unit.interface.expect("unit interface");
    let (program, errors) = parse(&format!(
        "program T; uses Domain as D; {TYPES} begin {body} end."
    ));
    assert!(errors.is_empty(), "{errors:#?}");
    crate::analyze_program_with_interfaces(&program, &[interface])
        .expect("program analysis")
        .errors
}

#[test]
fn conversion_arguments_reject_type_designators_in_all_name_forms() {
    for (target, argument) in [
        ("Small", "Small"),
        ("Small", "(Small)"),
        ("Small", "SmallAlias"),
        ("SmallAlias", "Small"),
        ("D.Small", "D.Small"),
        ("D.Small", "(D.Small)"),
        ("D.Small", "D.RangeAlias"),
        ("UserId", "UserId"),
        ("UserId", "(UserId)"),
        ("UserId", "IdAlias"),
        ("IdAlias", "UserId"),
        ("D.UserId", "D.UserId"),
        ("D.UserId", "(D.UserId)"),
        ("D.UserId", "D.IdAlias"),
        ("integer", "UserId"),
        ("integer", "D.UserId"),
        ("Small", "integer"),
        ("UserId", "integer"),
    ] {
        let errors = analyze(&format!("const Value := {target}({argument});"));
        assert_eq!(errors.len(), 1, "{target}({argument}): {errors:#?}");
        assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{errors:#?}");
        assert!(errors[0].message.contains("type name"), "{errors:#?}");
        assert!(
            errors[0]
                .help
                .as_deref()
                .is_some_and(|help| help.contains(&format!("{target}(Value)"))),
            "{errors:#?}"
        );
    }
}

#[test]
fn conversion_arguments_reject_type_names_inside_computed_values() {
    for value in [
        "Small(if true then Small else Small(1) end if)",
        "Small(Small + 1)",
        "Small(-Small)",
        "UserId(if true then UserId else UserId(1) end if)",
        "integer(if true then D.UserId else D.UserId(1) end if)",
    ] {
        let errors = analyze(&format!("const Value := {value};"));
        assert_eq!(errors.len(), 1, "{value}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{errors:#?}");
        assert!(errors[0].message.contains("type name"), "{errors:#?}");
    }
}

#[test]
fn conversion_arguments_accept_values_and_preserve_type_membership() {
    let errors = analyze(
        "const A := SmallAlias((3)); const B: UserId := IdAlias(2);
         const Raw: integer := integer(B); const Q := D.Small(D.Number);
         const Id: D.UserId := D.IdAlias(D.Id); const Present: boolean := A in Small;
         const Qualified: boolean := Q in (D.Small); var Stored: Small := Small(2);
         const Copy := Small(Stored);",
    );
    assert!(errors.is_empty(), "{errors:#?}");
}

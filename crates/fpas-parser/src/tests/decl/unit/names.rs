use super::*;

#[test]
fn minimal_unit() {
    let unit = parse_unit_ok("unit MyApp.Core;\nend unit;");
    assert_eq!(unit.name.parts, vec!["MyApp", "Core"]);
    assert!(unit.uses.is_empty());
    assert!(unit.declarations.is_empty());
}

#[test]
fn single_segment_unit_name() {
    let unit = parse_unit_ok("unit Utils;\nend unit;");
    assert_eq!(unit.name.parts, vec!["Utils"]);
}

#[test]
fn deeply_qualified_unit_name() {
    let unit = parse_unit_ok("unit App.Sub.Module.Deep;\nend unit;");
    assert_eq!(unit.name.parts, vec!["App", "Sub", "Module", "Deep"]);
}

#[test]
fn keyword_cannot_be_a_unit_name_segment() {
    let (_, errors) = parse_compilation_unit_with_errors("unit array.Plugin;\nend unit;");
    assert!(!errors.is_empty());
}

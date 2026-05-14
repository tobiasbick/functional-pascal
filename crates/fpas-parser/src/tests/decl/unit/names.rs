use super::*;

#[test]
fn minimal_unit() {
    let unit = parse_unit_ok("unit MyApp.Core;");
    assert_eq!(unit.name.parts, vec!["MyApp", "Core"]);
    assert!(unit.uses.is_empty());
    assert!(unit.declarations.is_empty());
}

#[test]
fn single_segment_unit_name() {
    let unit = parse_unit_ok("unit Utils;");
    assert_eq!(unit.name.parts, vec!["Utils"]);
}

#[test]
fn deeply_qualified_unit_name() {
    let unit = parse_unit_ok("unit App.Sub.Module.Deep;");
    assert_eq!(unit.name.parts, vec!["App", "Sub", "Module", "Deep"]);
}

#[test]
fn unit_name_leading_std_unit_keyword_segment() {
    let unit = parse_unit_ok("unit array.Plugin;");
    assert_eq!(unit.name.parts, vec!["Array", "Plugin"]);
}
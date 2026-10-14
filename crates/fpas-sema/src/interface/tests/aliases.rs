//! Exported aliases retain nominal identity and qualified constructor lookup.

use super::*;

#[test]
fn imported_enum_aliases_register_variants_without_overwriting_the_generic_type() {
    let dependency = analyze_unit(
        &parse_unit(
            "unit Domain;
        public type Lookup of T = enum Found(Value: T); Missing; end enum;
        public type ZIntLookup = Lookup of integer;
        end unit;",
        ),
        &[],
    )
    .expect("dependency analysis")
    .interface
    .expect("dependency interface");
    for (imports, qualifier) in [
        ("Domain as D", "D."),
        ("Domain as DomainUnit", "DomainUnit."),
    ] {
        let consumer = parse_unit(&format!(
            "unit Consumer; uses {imports}; public procedure Run(); begin
            const Missing := {qualifier}ZIntLookup.Missing;
            const Fixed := {qualifier}ZIntLookup.Found(1);
            const Generic := {qualifier}Lookup.Found('text');
            const KeepMissing: {qualifier}Lookup of integer := Missing;
            const KeepFixed: {qualifier}Lookup of integer := Fixed;
            const KeepGeneric: {qualifier}Lookup of string := Generic;
            end procedure; end unit;"
        ));
        let analysis =
            analyze_unit(&consumer, std::slice::from_ref(&dependency)).expect("consumer analysis");
        assert!(
            analysis.metadata.errors.is_empty(),
            "{imports}: {:?}",
            analysis.metadata.errors
        );
    }
}

#[test]
fn reexported_record_alias_preserves_its_owner_across_facades() {
    let original = analyze_unit(
        &parse_unit(
            "unit Repro.Model;
public type Model = record
  public Value: integer;
  public Changed: Option of procedure(Value: integer) := None;
  function ReadValue(Self: Model): integer;
  begin return Self.Value; end function;
end record;\nend unit;",
        ),
        &[],
    )
    .expect("model analysis")
    .interface
    .expect("model interface");
    let original_type = original.symbols[0].ty.clone();
    let mut interfaces = vec![original];
    for (unit, dependency, target) in [
        ("Repro.Facade", "Repro.Model as Data", "Data.Model"),
        ("Repro.Second", "Repro.Facade as Data", "Data.Model"),
    ] {
        let analysis = analyze_unit(
            &parse_unit(&format!(
                "unit {unit}; uses {dependency}; public type Model = {target};\nend unit;"
            )),
            &interfaces,
        )
        .expect("facade analysis");
        assert!(
            analysis.metadata.errors.is_empty(),
            "{:?}",
            analysis.metadata.errors
        );
        let interface = analysis.interface.expect("facade interface");
        assert_eq!(
            interface.symbols[0].ty, original_type,
            "{unit} must preserve record metadata"
        );
        interfaces.push(interface);
    }
}

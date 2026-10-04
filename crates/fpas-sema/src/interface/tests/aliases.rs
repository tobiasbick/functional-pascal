//! Exported aliases retain the original record declaration identity.

use super::*;

#[test]
fn reexported_resource_alias_remains_non_equatable() {
    let original = analyze_unit(&parse_unit(
        "unit Demo.Resources; uses Std.Net as Net; public type Connection = Net.Connection; end unit;"
    ), &[]).expect("resource interface").interface.expect("valid interface");
    let InterfaceType::Record(record) = &original.symbols[0].ty else {
        panic!("nominal resource");
    };
    assert!(record.is_resource);
    let facade = analyze_unit(&parse_unit(
        "unit Demo.Facade; uses Demo.Resources as Resources; public type Connection = Resources.Connection; end unit;"
    ), &[original]).expect("facade interface").interface.expect("valid interface");
    let consumer = analyze_unit(&parse_unit(
        "unit Demo.Consumer; uses Demo.Facade as Facade;\n         public function Same(A: option of (Facade.Connection); B: option of (Facade.Connection)): boolean;\n         begin return A = B; end function; end unit;"
    ), &[facade]).expect("consumer diagnostics");
    assert!(
        consumer
            .metadata
            .errors
            .iter()
            .any(|error| error.message.contains("Equality requires")),
        "{:#?}",
        consumer.metadata.errors
    );
}

#[test]
fn reexported_record_alias_preserves_its_owner_across_facades() {
    let original = analyze_unit(
        &parse_unit(
            r#"unit Repro.Model;
  public type Model = record
  public Value: integer;
  public Changed: Option of (procedure(Value: integer)) := Option.None;
end record;
end unit;
"#,
        ),
        &[],
    )
    .expect("model analysis")
    .interface
    .expect("model interface");
    let original_type = original.symbols[0].ty.clone();
    let mut interfaces = vec![original];
    for (unit, dependency, _target) in [
        ("Repro.Facade", "Repro.Model", "Repro.Model.Model"),
        ("Repro.Second", "Repro.Facade", "Repro.Facade.Model"),
    ] {
        let analysis = analyze_unit(
            &parse_unit(&format!(
                "unit {unit}; uses {dependency} as Imported; public type Model = Imported.Model; end unit;"
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

//! Exported aliases retain the original record declaration identity.

use super::*;

#[test]
fn reexported_record_alias_preserves_its_owner_across_facades() {
    let original = analyze_unit(
        &parse_unit(
            "unit Repro.Model;
public type Model = record
  public Value: integer;
  function ReadChanged(Self: Model): Option of procedure(Value: integer);
  begin return None end;
  procedure WriteChanged(Self: Model; Handler: Option of procedure(Value: integer));
  begin end;
  public event Changed: procedure(Value: integer) read ReadChanged write WriteChanged;
end;",
        ),
        &[],
    )
    .expect("model analysis")
    .interface
    .expect("model interface");
    let original_type = original.symbols[0].ty.clone();
    let mut interfaces = vec![original];
    for (unit, dependency, target) in [
        ("Repro.Facade", "Repro.Model", "Repro.Model.Model"),
        ("Repro.Second", "Repro.Facade", "Repro.Facade.Model"),
    ] {
        let analysis = analyze_unit(
            &parse_unit(&format!(
                "unit {unit}; uses {dependency}; public type Model = {target};"
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

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

fn native_keys() -> fpas_unit::interface::UnitInterface {
    let analysis = analyze_unit(
        &parse_unit(
            "unit NativeKeys; uses Std.Console as Console;
          public type Kind = Console.KeyKind;
          public type Key = Console.KeyEvent;
          end unit;",
        ),
        &[],
    )
    .expect("native facade analysis");
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
    let interface = analysis.interface.expect("native facade interface");
    fpas_unit::interface::decode_interface(
        &fpas_unit::interface::encode_interface(&interface).expect("encode"),
    )
    .expect("decode")
}

#[test]
fn native_enum_aliases_resolve_record_fields_without_importing_the_native_unit() {
    let first = native_keys();
    let middle = analyze_unit(
        &parse_unit(
            "unit Middle; uses NativeKeys as Keys;
          public type Kind = Keys.Kind; public type Key = Keys.Key; end unit;",
        ),
        std::slice::from_ref(&first),
    )
    .expect("second facade analysis");
    assert!(
        middle.metadata.errors.is_empty(),
        "{:?}",
        middle.metadata.errors
    );
    let second = middle.interface.expect("second facade interface");
    for interface in [first, second] {
        let consumer = parse_unit(&format!(
            "unit Consumer; uses {} as Keys;
          public function IsEnter(Key: Keys.Key): boolean;
          begin return Key.kind = Keys.Kind.Enter; end function;
          public function IsEscape(Kind: Keys.Kind): boolean;
          begin return Kind = Keys.Kind.Escape; end function;
          end unit;",
            interface.unit_name
        ));
        let analysis = analyze_unit(&consumer, &[interface]).expect("consumer analysis");
        assert!(
            analysis.metadata.errors.is_empty(),
            "{:?}",
            analysis.metadata.errors
        );
    }
}

#[test]
fn native_enum_alias_metadata_does_not_expose_native_names_or_merge_other_enums() {
    let interface = native_keys();
    for declarations in [
        "public function Check(Key: Keys.Key): boolean; begin return Key.kind = Other.Enter; end function;",
        "public type Leak = Std.Console.KeyKind;",
        "public type Leak = Console.KeyKind;",
    ] {
        let consumer = parse_unit(&format!(
            "unit Consumer; uses NativeKeys as Keys;
          type Other = enum Enter; Escape; end enum;
          {declarations} end unit;"
        ));
        let analysis =
            analyze_unit(&consumer, std::slice::from_ref(&interface)).expect("consumer analysis");
        assert!(
            !analysis.metadata.errors.is_empty(),
            "accepted {declarations}"
        );
        if declarations.contains("Other.Enter") {
            assert!(
                analysis
                    .metadata
                    .errors
                    .iter()
                    .any(|error| error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH),
                "{:?}",
                analysis.metadata.errors
            );
        }
    }
}

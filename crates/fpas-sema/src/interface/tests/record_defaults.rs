//! Scalar default expressions survive public interface extraction.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`.

use fpas_unit::interface::{ConstantValue as Value, FieldDefault, InterfaceType, SymbolKind};

use super::{analyze_unit, parse_unit};

#[test]
fn exported_record_defaults_fold_scalar_operators_and_constant_references() {
    let unit = parse_unit(
        "unit Demo.Defaults;
         const Base: integer := 1 + 2;
         public const Scale: real := Base * 1.0;
         public type Settings = record
           public Count: integer := -(Base * 2) + 20 div 3 mod 4;
           public Scale: real := Scale / 2 + 0.5;
           public Label: string := 'de' + 'fault';
           public Enabled: boolean := (not false and ('a' < 'b')) xor false;
           public ShortCircuit: boolean := true or (1 div 0 = 0);
         end record;
         end unit;",
    );
    let analysis = analyze_unit(&unit, &[]).expect("interface extraction");
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:#?}",
        analysis.metadata.errors
    );
    let interface = analysis.interface.expect("valid interface");
    let settings = interface
        .symbols
        .iter()
        .find(|symbol| symbol.name == "Settings")
        .expect("Settings");
    let InterfaceType::Record(settings) = &settings.ty else {
        panic!("record");
    };
    assert_eq!(
        settings
            .fields
            .iter()
            .map(|field| field.default_value.clone())
            .collect::<Vec<_>>(),
        [
            Some(Value::Integer(-4)),
            Some(Value::Real(2.0_f64.to_bits())),
            Some(Value::String("default".into())),
            Some(Value::Boolean(true)),
            Some(Value::Boolean(true))
        ]
        .map(|value| value.map(FieldDefault::Constant))
    );
    let scale = interface
        .symbols
        .iter()
        .find(|symbol| symbol.name == "Scale")
        .expect("Scale");
    assert_eq!(
        scale.kind,
        SymbolKind::Constant(Some(Value::Real(3.0_f64.to_bits())))
    );
}

#[test]
fn exported_record_defaults_resolve_import_aliases_and_private_constants() {
    let dependency = parse_unit("unit Demo.Base; public const Value: integer := 1 + 2; end unit;");
    let interface = analyze_unit(&dependency, &[])
        .expect("dependency")
        .interface
        .expect("interface");
    let unit = parse_unit(
        "unit Demo.Defaults; uses Demo.Base as Base;
         const Local: integer := bAsE.vAlUe * 2;
         public type Settings = record public Count: integer := Local + 1; end record;
         end unit;",
    );
    let analysis = analyze_unit(&unit, &[interface]).expect("consumer interface");
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:#?}",
        analysis.metadata.errors
    );
    let interface = analysis.interface.expect("interface");
    let InterfaceType::Record(settings) = &interface.symbols[0].ty else {
        panic!("record");
    };
    assert_eq!(
        settings.fields[0].default_value,
        Some(FieldDefault::Constant(Value::Integer(7)))
    );
}

#[test]
fn wrong_record_default_type_is_a_semantic_diagnostic() {
    let unit = parse_unit(
        "unit Demo.Defaults;
         public type Settings = record public Count: integer := 'wrong' + 'type'; end record;
         end unit;",
    );
    let analysis = analyze_unit(&unit, &[]).expect("semantic analysis reports the invalid default");
    assert!(analysis.interface.is_none());
    assert!(
        analysis
            .metadata
            .errors
            .iter()
            .any(|error| error.message.contains("default value")),
        "{:#?}",
        analysis.metadata.errors
    );
}

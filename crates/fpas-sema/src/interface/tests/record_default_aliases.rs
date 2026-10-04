//! Scalar field defaults and visibility survive local and imported alias chains.
//!
//! **Documentation:** `docs/pascal/language/types/type-aliases.md`.

use super::*;
use fpas_unit::interface::{ConstantValue, FieldDefault};

fn model() -> fpas_unit::interface::UnitInterface {
    analyze_unit(
        &parse_unit(
            "unit Demo.Model; const Base: integer := 1 + 2;
         public type Settings = record
           public Required: integer;
           public Count: integer := Base * 2;
           public Label: string := 'de' + 'fault';
         end record;
         public type LocalAlias = Settings;
         public type SecondAlias = lOcAlAlIaS;
         end unit;",
        ),
        &[],
    )
    .expect("model analysis")
    .interface
    .expect("model interface")
}

#[test]
fn local_aliases_export_the_original_defaults_and_required_fields() {
    let interface = model();
    let original = &interface
        .symbols
        .iter()
        .find(|symbol| symbol.name == "Settings")
        .unwrap()
        .ty;
    for symbol in &interface.symbols {
        assert_eq!(&symbol.ty, original, "{}", symbol.name);
        let InterfaceType::Record(record) = &symbol.ty else {
            panic!("record")
        };
        assert_eq!(record.fields[0].default_value, None);
        assert_eq!(
            record.fields[1].default_value,
            Some(FieldDefault::Constant(ConstantValue::Integer(6)))
        );
        assert_eq!(
            record.fields[2].default_value,
            Some(FieldDefault::Constant(ConstantValue::String(
                "default".into()
            )))
        );
    }
}

#[test]
fn facade_chains_preserve_defaults_without_importing_the_original_type() {
    let mut interface = model();
    let original = interface.symbols[0].ty.clone();
    for (name, dependency, target) in [
        ("Demo.Facade", "Demo.Model", "SecondAlias"),
        ("Demo.Second", "Demo.Facade", "Settings"),
    ] {
        let facade = parse_unit(&format!(
            "unit {name}; uses {dependency} as Imported;
             public type Settings = iMpOrTeD.{target}; end unit;"
        ));
        let analysis = analyze_unit(&facade, &[interface]).expect("facade analysis");
        assert!(
            analysis.metadata.errors.is_empty(),
            "{:?}",
            analysis.metadata.errors
        );
        interface = analysis.interface.expect("facade interface");
        assert_eq!(interface.symbols[0].ty, original);
    }
    let consumer = parse_unit(
        "unit Demo.Consumer; uses Demo.Second as Facade;\n         public function Make(): Facade.Settings;\n         begin return Facade.Settings(Required := 1); end function;\n         end unit;",
    );
    let analysis = analyze_unit(&consumer, &[interface]).expect("consumer analysis");
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
}

#[test]
fn collection_aliases_preserve_and_install_embedded_record_defaults() {
    let facade = parse_unit(
        "unit Demo.Lists; uses Demo.Model as Model;\n         public type Settings = Model.Settings;\n         public type SettingsList = array of (Settings);\n         public type NestedList = array of (SettingsList);\n         end unit;",
    );
    let interface = analyze_unit(&facade, &[model()])
        .expect("facade analysis")
        .interface
        .unwrap();
    let consumer = parse_unit(
        "unit Demo.Consumer; uses Demo.Lists as Lists;\n         public function Make(): Lists.SettingsList;\n         begin return [Lists.Settings(Required := 7)]; end function;\n         end unit;",
    );
    let analysis = analyze_unit(&consumer, &[interface]).expect("consumer analysis");
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
    assert!(
        analysis
            .metadata
            .record_defaults
            .contains_key("demo.model.settings")
    );
}

#[test]
fn supporting_type_defaults_work_in_imported_callable_argument_context() {
    let original = model();
    let facade = parse_unit(
        "unit Demo.Api; uses Demo.Model as Model;\n         public type Settings = Model.Settings;\n         public function CountOf(Value: Settings): integer;\n         begin return Value.Count; end function; end unit;",
    );
    let interface = analyze_unit(&facade, &[original.clone()])
        .unwrap()
        .interface
        .unwrap();
    let consumer = parse_unit(
        "unit Demo.Consumer; uses Demo.Api as Api;\n         public function CountOf(): integer;\n         begin return Api.CountOf(Api.Settings(Required := 7)); end function; end unit;",
    );
    let analysis = crate::analyze_unit_with_interface_support(
        &consumer,
        &[interface.clone()],
        &[original, interface],
    )
    .expect("consumer with supporting type");
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
}

#[test]
fn defaulted_private_fields_remain_inaccessible_through_an_alias() {
    let original = analyze_unit(
        &parse_unit(
            "unit Demo.Secret;
         public type Settings = record Count: integer := 3; end record;
         end unit;",
        ),
        &[],
    )
    .unwrap()
    .interface
    .unwrap();
    let facade = analyze_unit(
        &parse_unit(
            "unit Demo.Facade; uses Demo.Secret as Secret;
         public type Settings = Secret.Settings; end unit;",
        ),
        &[original],
    )
    .unwrap()
    .interface
    .unwrap();
    let InterfaceType::Record(record) = &facade.symbols[0].ty else {
        panic!("record")
    };
    assert_eq!(
        record.fields[0].default_value,
        Some(FieldDefault::Constant(ConstantValue::Integer(3)))
    );
    assert_eq!(record.owner_unit.as_deref(), Some("demo.secret"));
    assert_eq!(record.private_members, ["count"]);
    let analysis = analyze_unit(
        &parse_unit(
            "unit Demo.Consumer; uses Demo.Facade as Facade;\n         public function Make(): Facade.Settings;\n         begin return Facade.Settings(); end function; end unit;",
        ),
        &[facade],
    )
    .unwrap();
    assert!(!analysis.metadata.errors.is_empty());
    assert!(
        analysis
            .metadata
            .errors
            .iter()
            .all(|error| error.code != fpas_diagnostics::codes::SEMA_MISSING_RECORD_FIELD)
    );
}

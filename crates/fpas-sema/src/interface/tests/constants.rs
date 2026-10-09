use super::*;
use fpas_unit::interface::{ConstantValue, decode_interface, encode_interface};

fn interface(
    source: &str,
    dependencies: &[fpas_unit::interface::UnitInterface],
) -> fpas_unit::interface::UnitInterface {
    let analysis = analyze_unit(&parse_unit(source), dependencies).expect("unit analysis");
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:?}",
        analysis.metadata.errors
    );
    let interface = analysis.interface.expect("interface");
    decode_interface(&encode_interface(&interface).expect("encode")).expect("decode")
}

#[test]
fn constant_classification_and_values_survive_transitive_unit_interfaces() {
    let first = interface(
        "unit Original;
        function ReadValue(): integer; begin return 40; end function;
        public const Fixed: integer := 6 * 7;
        public const Dynamic: integer := ReadValue();
        public const Values: array of integer := [1, 2];
        end unit;",
        &[],
    );
    let second = interface(
        "unit Facade; uses Original;
        public const Fixed: integer := Original.Fixed + 1;
        public const Dynamic: integer := Original.Dynamic + 2;
        public const Values: array of integer := Original.Values;
        end unit;",
        &[first],
    );
    let kind = |name: &str| {
        &second
            .symbols
            .iter()
            .find(|symbol| symbol.name == name)
            .expect("symbol")
            .kind
    };
    assert_eq!(
        kind("Fixed"),
        &SymbolKind::Constant(Some(ConstantValue::Integer(43)))
    );
    assert_eq!(kind("Dynamic"), &SymbolKind::ComputedConstant);
    assert_eq!(kind("Values"), &SymbolKind::Constant(None));
    let consumer = analyze_unit(
        &parse_unit(
            "unit Consumer; uses Facade;
        procedure Check(); begin case 43 of when Facade.Fixed: null; end case;
        case 42 of when Facade.Dynamic: null; end case; end procedure; end unit;",
        ),
        &[second],
    )
    .expect("consumer");
    assert!(
        consumer.metadata.errors.iter().any(|error| error.code
            == fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION
            && error.message.contains("Facade.Dynamic")),
        "{:?}",
        consumer.metadata.errors
    );
}

#[test]
fn scalar_exports_fold_operators_and_owned_enum_values() {
    let interface = interface(
        "unit Values;
        public type State = enum Ready; end enum;
        public const Current: State := State.Ready;
        public const Number: real := 42.0;
        public const Ratio: real := 5 / 2;
        public const Caption: string := 'Hello' + ' World';
        public const Flag: boolean := not false and (3 < 4);
        public const Wrapped: integer := 9223372036854775807 + 1;
        end unit;",
        &[],
    );
    let value = |name: &str| {
        interface
            .symbols
            .iter()
            .find(|symbol| symbol.name == name)
            .expect("symbol")
            .kind
            .clone()
    };
    assert_eq!(
        value("Number"),
        SymbolKind::Constant(Some(ConstantValue::Real(42f64.to_bits())))
    );
    assert_eq!(
        value("Ratio"),
        SymbolKind::Constant(Some(ConstantValue::Real(2.5f64.to_bits())))
    );
    assert_eq!(
        value("Caption"),
        SymbolKind::Constant(Some(ConstantValue::String("Hello World".into())))
    );
    assert_eq!(
        value("Flag"),
        SymbolKind::Constant(Some(ConstantValue::Boolean(true)))
    );
    assert_eq!(
        value("Wrapped"),
        SymbolKind::Constant(Some(ConstantValue::Integer(i64::MIN)))
    );
    assert_eq!(
        value("Current"),
        SymbolKind::Constant(Some(ConstantValue::EnumValue {
            enum_name: "values.state".into(),
            variant_name: "ready".into(),
            backing_value: 0
        }))
    );
}

#[test]
fn record_constants_preserve_nested_fields_and_enum_identity_across_units() {
    let original = interface(
        "unit Demo.Original;
        public type Color = enum Red; Green; end enum;
        public type Flags = record public Enabled: boolean; public Color: Color; end record;
        public type Settings = record public Flags: Flags; public Values: array of integer; end record;
        public const Config: Settings := Settings(Flags := Flags(Enabled := true, Color := Color.Red), Values := [1]);
        end unit;", &[],
    );
    let facade = interface(
        "unit Demo.Facade; uses Demo.Original as O;
        public const Copy: O.Settings := O.Config;
        public const Enabled: boolean := Copy.Flags.Enabled;
        public const Selected: O.Color := Copy.Flags.Color;
        end unit;",
        std::slice::from_ref(&original),
    );
    let copy = facade
        .symbols
        .iter()
        .find(|symbol| symbol.name == "Copy")
        .expect("copy");
    assert_eq!(copy.kind, SymbolKind::Constant(None));
    let fields = &copy.constant_record.as_ref().expect("record values").fields;
    assert!(
        !fields.contains_key("values"),
        "non-scalar aggregates stay runtime values"
    );
    let selected = facade
        .symbols
        .iter()
        .find(|symbol| symbol.name == "Selected")
        .expect("enum projection");
    assert_eq!(
        selected.kind,
        SymbolKind::Constant(Some(ConstantValue::EnumValue {
            enum_name: "demo.original.color".into(),
            variant_name: "red".into(),
            backing_value: 0,
        }))
    );
    let consumer = analyze_unit(&parse_unit(
        "unit Consumer; uses Demo.Original as O, Demo.Facade as F;
        procedure Check(B: Option of boolean; C: Option of O.Color);
        begin case B of when Some(F.Copy.Flags.Enabled): null; when Some(false): null; when None: null; end case;
        case C of when Some(F.Selected): null; when Some(O.Color.Green): null; when None: null; end case;
        end procedure; end unit;"
    ), &[original, facade]).expect("consumer");
    assert!(
        consumer.metadata.errors.is_empty(),
        "{:#?}",
        consumer.metadata.errors
    );
}

#[test]
fn imported_record_patterns_detect_duplicates_and_reject_computed_fields() {
    let original = interface(
        "unit Flags;
        public type Config = record public Enabled: boolean; end record;
        function ReadFlag(): boolean; begin return true; end function;
        public const Fixed: Config := Config(Enabled := true);
        public const Dynamic: Config := Config(Enabled := ReadFlag()); end unit;",
        &[],
    );
    assert!(
        original
            .symbols
            .iter()
            .find(|symbol| symbol.name == "Dynamic")
            .expect("dynamic")
            .constant_record
            .is_none()
    );
    for (label, rest, expected) in [
        (
            "Flags.Fixed.Enabled",
            "when Some(true): null; when Some(false): null;",
            fpas_diagnostics::codes::SEMA_UNREACHABLE_CASE_LABEL,
        ),
        (
            "Flags.Dynamic.Enabled",
            "when Some(_): null;",
            fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION,
        ),
    ] {
        let source = format!(
            "unit Consumer; uses Flags;
            procedure Check(B: Option of boolean); begin case B of
            when Some({label}): null; {rest} when None: null; end case; end procedure; end unit;"
        );
        let consumer =
            analyze_unit(&parse_unit(&source), std::slice::from_ref(&original)).expect("consumer");
        assert!(
            consumer
                .metadata
                .errors
                .iter()
                .any(|error| error.code == expected),
            "{:#?}",
            consumer.metadata.errors
        );
    }
}

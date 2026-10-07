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

//! Aggregate constant metadata preserves data, canonical names and dependency hashes.

use fpas_unit::interface::{
    ConstantValue, InterfaceSymbol, InterfaceType, StaticValue, SymbolKind, UnitInterface,
    decode_interface, encode_interface,
};

fn sample() -> UnitInterface {
    let integer = |value| StaticValue::Scalar(ConstantValue::Integer(value));
    UnitInterface {
        unit_name: "Demo.Values".into(),
        symbols: vec![InterfaceSymbol {
            name: "Data".into(),
            qualified_name: "Demo.Values.Data".into(),
            ty: InterfaceType::Named("Demo.Values.RecordData".into()),
            kind: SymbolKind::AggregateConstant(StaticValue::Record(vec![
                (
                    "Numbers".into(),
                    StaticValue::Array(vec![integer(i64::MIN), integer(i64::MAX)]),
                ),
                (
                    "Mapping".into(),
                    StaticValue::Dictionary(vec![(
                        StaticValue::Scalar(ConstantValue::String("key".into())),
                        integer(42),
                    )]),
                ),
                (
                    "Present".into(),
                    StaticValue::OptionSome(Box::new(integer(1))),
                ),
                ("Absent".into(), StaticValue::OptionNone),
                (
                    "Success".into(),
                    StaticValue::ResultOk(Box::new(integer(2))),
                ),
                (
                    "Failure".into(),
                    StaticValue::ResultError(Box::new(integer(3))),
                ),
                (
                    "Choice".into(),
                    StaticValue::EnumVariant("Empty".into(), Vec::new()),
                ),
                (
                    "Nan".into(),
                    StaticValue::Scalar(ConstantValue::Real(0x7ff8_0000_0000_0001)),
                ),
            ])),
        }],
    }
}

#[test]
fn aggregate_metadata_round_trips_every_static_data_shape() {
    let source = sample();
    let encoded = encode_interface(&source).expect("encode aggregate values");
    let decoded = decode_interface(&encoded).expect("decode aggregate values");
    assert_eq!(decoded, source.canonicalized());
    assert_eq!(
        encode_interface(&decoded).expect("canonical encoding"),
        encoded
    );
}

#[test]
fn static_value_changes_invalidate_dependents_and_names_are_canonical() {
    let original = sample();
    let mut changed = sample();
    if let SymbolKind::AggregateConstant(StaticValue::Record(fields)) = &mut changed.symbols[0].kind
    {
        fields[0].0.make_ascii_lowercase();
    } else {
        panic!("static record fixture");
    }
    assert_eq!(
        original.digest().expect("original hash"),
        changed.digest().expect("name hash")
    );
    if let SymbolKind::AggregateConstant(StaticValue::Record(fields)) = &mut changed.symbols[0].kind
    {
        fields[0].1 = StaticValue::Array(vec![StaticValue::Scalar(ConstantValue::Integer(0))]);
    } else {
        panic!("static record fixture");
    }
    assert_ne!(
        original.digest().expect("original hash"),
        changed.digest().expect("changed data hash")
    );
}

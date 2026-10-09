//! Canonical encoding and hashing of known record-field constants.
//! See `docs/pascal/language/basics/constants.md`.

use super::*;
use fpas_unit::interface::{RecordConstant, RecordConstantField};
use std::collections::BTreeMap;
use std::sync::Arc;

fn fixture() -> UnitInterface {
    let mut interface = sample_interface();
    let flags = Arc::new(RecordConstant {
        fields: BTreeMap::from([
            (
                "Enabled".into(),
                RecordConstantField::Scalar(ConstantValue::Boolean(true)),
            ),
            (
                "Color".into(),
                RecordConstantField::Scalar(ConstantValue::EnumValue {
                    enum_name: "Demo.Api.Color".into(),
                    variant_name: "Red".into(),
                    backing_value: 0,
                }),
            ),
        ]),
    });
    interface.symbols.push(InterfaceSymbol {
        name: "Config".into(),
        qualified_name: "Demo.Api.Config".into(),
        ty: InterfaceType::Named("Demo.Api.Settings".into()),
        kind: SymbolKind::Constant(None),
        discard: Default::default(),
        constant_record: Some(Arc::new(RecordConstant {
            fields: BTreeMap::from([
                (
                    "Left".into(),
                    RecordConstantField::Record(Arc::clone(&flags)),
                ),
                ("Right".into(), RecordConstantField::Record(flags)),
            ]),
        })),
    });
    interface
}

#[test]
fn record_values_are_canonical_and_round_trip_without_changing_runtime_kind() {
    let canonical = fixture().canonicalized();
    let symbol = canonical
        .symbols
        .iter()
        .find(|symbol| symbol.name == "Config")
        .expect("config");
    assert_eq!(symbol.kind, SymbolKind::Constant(None));
    let fields = &symbol.constant_record.as_ref().expect("record").fields;
    let (RecordConstantField::Record(left), RecordConstantField::Record(right)) =
        (&fields["left"], &fields["right"])
    else {
        panic!("nested fields");
    };
    assert!(
        Arc::ptr_eq(left, right),
        "canonicalization preserves shared values"
    );
    assert_eq!(
        left.fields["color"],
        RecordConstantField::Scalar(ConstantValue::EnumValue {
            enum_name: "demo.api.color".into(),
            variant_name: "red".into(),
            backing_value: 0,
        })
    );
    let encoded = encode_interface(&fixture()).expect("encode");
    assert_eq!(decode_interface(&encoded).expect("decode"), canonical);
    assert_eq!(
        encoded,
        encode_interface(&canonical).expect("canonical bytes")
    );
}

#[test]
fn known_field_values_participate_in_interface_hashing() {
    let original = fixture().canonicalized();
    let mut changed = original.clone();
    let symbol = changed
        .symbols
        .iter_mut()
        .find(|symbol| symbol.name == "Config")
        .expect("config");
    let record = Arc::make_mut(symbol.constant_record.as_mut().expect("record"));
    record.fields.insert(
        "left".into(),
        RecordConstantField::Scalar(ConstantValue::Boolean(false)),
    );
    assert_ne!(
        original.digest().expect("original digest"),
        changed.digest().expect("changed digest")
    );
}

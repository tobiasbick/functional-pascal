#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "interface fixtures use expect for compact round-trip assertions"
)]

use fpas_unit::interface::{
    CallableType, ConstantValue, EnumType, EnumVariant, FieldType, GenericParameter,
    InterfaceSymbol, InterfaceType, MethodType, ParameterType, RecordType, SymbolKind,
    TypeConstraint, UnitInterface, decode_interface, encode_interface,
};

fn sample_interface() -> UnitInterface {
    UnitInterface {
        unit_name: "Demo.Api".to_string(),
        symbols: vec![
            InterfaceSymbol {
                name: "Transform".to_string(),
                qualified_name: "Demo.Api.Transform".to_string(),
                ty: InterfaceType::Function(CallableType {
                    type_parameters: vec![GenericParameter {
                        name: "T".to_string(),
                        constraint: Some(TypeConstraint::Comparable),
                    }],
                    parameters: vec![ParameterType {
                        name: "Value".to_string(),
                        mutable: true,
                        ty: InterfaceType::GenericParameter(
                            "T".to_string(),
                            Some(TypeConstraint::Comparable),
                        ),
                    }],
                    result: Some(Box::new(InterfaceType::Array(Box::new(
                        InterfaceType::GenericParameter(
                            "T".to_string(),
                            Some(TypeConstraint::Comparable),
                        ),
                    )))),
                    variadic: false,
                }),
                kind: SymbolKind::Function,
            },
            InterfaceSymbol {
                name: "State".to_string(),
                qualified_name: "Demo.Api.State".to_string(),
                ty: InterfaceType::Enum(Box::new(EnumType {
                    name: "Demo.Api.State".to_string(),
                    variants: vec![
                        EnumVariant {
                            name: "Idle".to_string(),
                            fields: Vec::new(),
                            backing_value: Some(0),
                        },
                        EnumVariant {
                            name: "Failed".to_string(),
                            fields: vec![FieldType {
                                name: "Message".to_string(),
                                ty: InterfaceType::String,
                                default_value: None,
                            }],
                            backing_value: None,
                        },
                    ],
                })),
                kind: SymbolKind::Type,
            },
            InterfaceSymbol {
                name: "Limit".to_string(),
                qualified_name: "Demo.Api.Limit".to_string(),
                ty: InterfaceType::Integer,
                kind: SymbolKind::Constant(Some(ConstantValue::Integer(10))),
            },
            InterfaceSymbol {
                name: "Counter".to_string(),
                qualified_name: "Demo.Api.Counter".to_string(),
                ty: InterfaceType::Record(Box::new(RecordType {
                    name: "Demo.Api.Counter".to_string(),
                    owner_unit: Some("Demo.Api".to_string()),
                    private_members: vec!["CreateHidden".to_string(), "Value".to_string()],
                    fields: vec![FieldType {
                        name: "Value".to_string(),
                        ty: InterfaceType::Integer,
                        default_value: Some(ConstantValue::Integer(0)),
                    }],
                    methods: Vec::new(),
                    static_routines: vec![MethodType {
                        name: "CreateHidden".to_string(),
                        callable: CallableType {
                            type_parameters: Vec::new(),
                            parameters: Vec::new(),
                            result: Some(Box::new(InterfaceType::Named(
                                "Demo.Api.Counter".to_string(),
                            ))),
                            variadic: false,
                        },
                    }],
                    properties: Vec::new(),
                    events: Vec::new(),
                })),
                kind: SymbolKind::Type,
            },
        ],
    }
}

#[test]
fn semantic_interface_round_trip_preserves_every_shape() {
    let expected = sample_interface().canonicalized();
    let bytes = encode_interface(&expected).expect("interface encoding");
    let decoded = decode_interface(&bytes).expect("interface decoding");
    assert_eq!(decoded, expected);
}

#[test]
fn symbol_declaration_order_does_not_change_canonical_bytes_or_hash() {
    let left = sample_interface();
    let mut right = sample_interface();
    right.symbols.reverse();
    assert_eq!(
        encode_interface(&left).expect("left encoding"),
        encode_interface(&right).expect("right encoding")
    );
    assert_eq!(
        left.digest().expect("left digest"),
        right.digest().expect("right digest")
    );
}

#[test]
fn canonical_identity_case_does_not_change_bytes_or_hash() {
    let left = sample_interface();
    let mut right = sample_interface();
    let InterfaceType::Enum(enum_ty) = &mut right.symbols[1].ty else {
        panic!("expected enum fixture");
    };
    enum_ty.name.make_ascii_lowercase();
    let InterfaceType::Record(record) = &mut right.symbols[3].ty else {
        panic!("expected record fixture");
    };
    record.name.make_ascii_lowercase();
    record.owner_unit = Some("demo.api".to_string());

    assert_eq!(
        encode_interface(&left).expect("left encoding"),
        encode_interface(&right).expect("right encoding")
    );
    assert_eq!(
        left.digest().expect("left digest"),
        right.digest().expect("right digest")
    );
}

#[test]
fn observable_signature_and_value_changes_change_hash() {
    let original = sample_interface();
    let mut signature = sample_interface();
    signature.symbols[0].ty = InterfaceType::Procedure(CallableType {
        type_parameters: Vec::new(),
        parameters: Vec::new(),
        result: None,
        variadic: false,
    });
    let mut value = sample_interface();
    value.symbols[2].kind = SymbolKind::Constant(Some(ConstantValue::Integer(11)));

    let original_hash = original.digest().expect("original digest");
    assert_ne!(original_hash, signature.digest().expect("signature digest"));
    assert_ne!(original_hash, value.digest().expect("value digest"));

    let mut visibility = sample_interface();
    let InterfaceType::Record(record) = &mut visibility.symbols[3].ty else {
        panic!("expected record fixture");
    };
    record.private_members.clear();
    assert_ne!(
        original_hash,
        visibility.digest().expect("visibility digest")
    );
}

#[test]
fn unknown_interface_enum_tag_is_rejected() {
    let invalid = br#"{"unit_name":"Demo","symbols":[{"name":"X","qualified_name":"Demo.X","ty":"Imaginary","kind":"Variable"}]}"#;
    assert!(decode_interface(invalid).is_err());
}

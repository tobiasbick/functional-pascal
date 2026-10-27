//! Error schemas and construction ABI remain essential semantic interface data.

use fpas_bytecode::{ErrorInstanceKey, NominalKind, TypeIdentity};
use fpas_unit::{
    equality::EqualityDeclaration,
    errors::{
        ErrorConstructorRequest, ErrorConstructorSignature, ErrorRoutineConstructors, ErrorSchema,
        ErrorSchemaField, ErrorSchemaVariant,
    },
    interface::{TypeConstraint, decode_interface, encode_interface},
};

#[test]
fn private_equality_evidence_round_trips_and_affects_interface_identity() {
    let mut interface = super::sample_interface();
    let private = EqualityDeclaration {
        identity: TypeIdentity::Nominal {
            kind: NominalKind::Record,
            declaration: "demo.api.privatebox".into(),
            arguments: vec![TypeIdentity::Parameter(0)],
        },
        parameters: 1,
        fields: vec![TypeIdentity::Parameter(0)],
    };
    interface.errors.equality_declarations.push(private.clone());
    assert!(!interface.errors.is_empty());
    let before = interface.digest().expect("equality evidence fingerprint");
    let decoded = decode_interface(&encode_interface(&interface).expect("equality evidence"))
        .expect("private field evidence round trip");
    assert_eq!(decoded.errors.equality_declarations, vec![private.clone()]);
    assert_eq!(decoded.symbols, interface.clone().canonicalized().symbols);
    interface.errors.equality_declarations[0].fields =
        vec![TypeIdentity::Option(Box::new(TypeIdentity::Parameter(0)))];
    assert_ne!(before, interface.digest().expect("changed private fields"));
    interface.errors.equality_declarations.push(private.clone());
    assert!(
        encode_interface(&interface).is_err(),
        "conflicting evidence"
    );
    interface.errors.equality_declarations = vec![private.clone(), private];
    assert!(encode_interface(&interface).is_err(), "duplicate evidence");
}

#[test]
fn messages_constraints_and_private_constructor_requirements_affect_interface_identity() {
    let mut interface = super::sample_interface();
    interface.errors.schemas.push(ErrorSchema {
        declaration: "demo.api.failure".to_owned(),
        display_name: "Demo.Api.Failure".to_owned(),
        parameters: vec![Some(TypeConstraint::Error)],
        variants: vec![ErrorSchemaVariant {
            name: "Failed".to_owned(),
            message: Some("Operation failed".to_owned()),
            cause: Some(0),
            fields: vec![ErrorSchemaField {
                name: "Cause".to_owned(),
                ty: TypeIdentity::Parameter(0),
            }],
        }],
    });
    interface.errors.routines.push(ErrorRoutineConstructors {
        routine: "demo.api.wrap".to_owned(),
        signature: ErrorConstructorSignature {
            type_parameters: 1,
            requirements: vec![ErrorConstructorRequest {
                instance: ErrorInstanceKey {
                    declaration: "demo.api.failure".to_owned(),
                    arguments: vec![TypeIdentity::Parameter(0)],
                },
                variant: 0,
            }],
        },
    });
    let before = interface.digest().expect("error interface hash");
    let decoded = decode_interface(&encode_interface(&interface).expect("encoded schema"))
        .expect("decoded schema");
    assert_eq!(
        decoded.errors.schemas[0].parameters,
        vec![Some(TypeConstraint::Error)]
    );
    assert_eq!(
        decoded.errors.routines[0].signature.requirements[0]
            .instance
            .arguments,
        vec![TypeIdentity::Parameter(0)]
    );
    interface.errors.schemas[0].variants[0].message = Some("Updated label".to_owned());
    assert_ne!(
        before,
        interface
            .digest()
            .expect("message-sensitive interface hash")
    );
    assert!(
        interface.errors.schemas[0]
            .instantiate(&ErrorInstanceKey {
                declaration: "demo.api.failure".to_owned(),
                arguments: vec![TypeIdentity::Integer]
            })
            .is_err()
    );
}

#[test]
fn deep_semantic_arguments_use_a_flat_bounded_wire_shape() {
    let mut argument = TypeIdentity::String;
    for _ in 0..63 {
        argument = TypeIdentity::Nominal {
            kind: NominalKind::Record,
            declaration: "demo.box".to_owned(),
            arguments: vec![argument],
        };
    }
    let key = ErrorInstanceKey {
        declaration: "demo.failure".to_owned(),
        arguments: vec![argument],
    };
    key.validate(0).expect("bounded type graph");
    let bytes = serde_json::to_vec(&key).expect("flat encoded type key");
    let decoded: ErrorInstanceKey =
        serde_json::from_slice(&bytes).expect("flat decoding independent of recursive JSON limits");
    assert_eq!(decoded, key);
    let bad = br#"{"declaration":"demo.failure","arguments":[["String","Integer"]]}"#;
    assert!(serde_json::from_slice::<ErrorInstanceKey>(bad).is_err());
}

#[test]
fn callable_variadic_identity_survives_schema_substitution_and_wire_round_trips() {
    use fpas_bytecode::IdentityParameter;
    let callable = |variadic| TypeIdentity::Callable {
        parameters: vec![IdentityParameter {
            name: "values".to_owned(),
            mutable: false,
            ty: TypeIdentity::Parameter(0),
        }],
        result: Some(Box::new(TypeIdentity::Integer)),
        variadic,
    };
    let instantiate = |variadic| {
        let schema = ErrorSchema {
            declaration: "demo.data".to_owned(),
            display_name: "Demo.Data".to_owned(),
            parameters: vec![None],
            variants: vec![ErrorSchemaVariant {
                name: "Value".to_owned(),
                fields: vec![ErrorSchemaField {
                    name: "Value".to_owned(),
                    ty: TypeIdentity::Option(Box::new(callable(variadic))),
                }],
                message: None,
                cause: None,
            }],
        };
        schema
            .instantiate(&ErrorInstanceKey {
                declaration: "demo.data".to_owned(),
                arguments: vec![TypeIdentity::String],
            })
            .expect("callable schema substitution")
            .variants[0]
            .fields[0]
            .ty
            .clone()
    };
    let fixed = instantiate(false);
    let variadic = instantiate(true);
    let keys = [fixed, variadic].map(|argument| ErrorInstanceKey {
        declaration: "demo.phantom".to_owned(),
        arguments: vec![argument],
    });
    assert_ne!(keys[0], keys[1]);
    assert_ne!(
        fpas_unit::errors::instance_symbol(&keys[0]).expect("fixed symbol"),
        fpas_unit::errors::instance_symbol(&keys[1]).expect("variadic symbol")
    );
    for key in keys {
        let bytes = serde_json::to_vec(&key).expect("callable key wire");
        assert_eq!(
            serde_json::from_slice::<ErrorInstanceKey>(&bytes).expect("callable key round trip"),
            key
        );
    }
}

#[test]
fn schema_expansion_checks_the_combined_graph_before_copying_arguments() {
    let schema = ErrorSchema {
        declaration: "demo.wide".to_owned(),
        display_name: "Demo.Wide".to_owned(),
        parameters: vec![Some(TypeConstraint::Printable)],
        variants: vec![ErrorSchemaVariant {
            name: "Failed".to_owned(),
            fields: (0..65)
                .map(|index| ErrorSchemaField {
                    name: format!("Value{index}"),
                    ty: TypeIdentity::Parameter(0),
                })
                .collect(),
            message: None,
            cause: None,
        }],
    };
    let key = |argument| ErrorInstanceKey {
        declaration: "demo.wide".to_owned(),
        arguments: vec![argument],
    };
    schema
        .instantiate(&key(TypeIdentity::Integer))
        .expect("bounded positive control");
    let wide = TypeIdentity::Nominal {
        kind: NominalKind::Record,
        declaration: "demo.phantom".to_owned(),
        arguments: vec![TypeIdentity::Integer; 1024],
    };
    assert_eq!(
        schema
            .instantiate(&key(wide))
            .expect_err("combined expansion exceeds graph limit"),
        "substituted error type exceeds its graph limit"
    );
}

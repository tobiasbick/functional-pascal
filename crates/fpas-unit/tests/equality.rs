//! Structural equality evidence for source-less native error instantiation.

#![allow(clippy::unwrap_used, reason = "fixtures use direct assertions")]

use fpas_bytecode::{NominalKind, TypeIdentity as T};
use fpas_unit::{
    equality::{EqualityDeclaration, EqualityDeclarations},
    errors::{ErrorInstanceKey, ErrorInstantiationBudget, ErrorSchema, ErrorSchemaVariant},
    interface::TypeConstraint,
};

fn nominal(name: &str, args: Vec<T>) -> T {
    T::Nominal {
        kind: NominalKind::Record,
        declaration: name.into(),
        arguments: args,
    }
}

#[test]
fn schema_instantiation_requires_complete_nominal_equality_evidence() {
    let point = nominal("test.point", vec![]);
    let node = nominal("test.node", vec![]);
    let declarations = EqualityDeclarations::new([
        EqualityDeclaration {
            identity: point.clone(),
            parameters: 0,
            fields: vec![T::Integer],
        },
        EqualityDeclaration {
            identity: node.clone(),
            parameters: 0,
            fields: vec![T::Option(Box::new(node.clone())), point.clone()],
        },
    ])
    .unwrap();
    let schema = ErrorSchema {
        declaration: "test.marker".into(),
        display_name: "test.marker".into(),
        parameters: vec![Some(TypeConstraint::Equatable)],
        variants: vec![ErrorSchemaVariant {
            name: "Failed".into(),
            fields: vec![],
            cause: None,
            message: None,
        }],
    };
    let key = ErrorInstanceKey {
        declaration: schema.declaration.clone(),
        arguments: vec![node],
    };
    assert!(
        schema.instantiate(&key).is_err(),
        "a nominal name is insufficient"
    );
    assert!(
        schema
            .instantiate_with_declarations(
                &key,
                &mut ErrorInstantiationBudget::default(),
                &declarations
            )
            .is_ok()
    );
    let bad = EqualityDeclarations::new([EqualityDeclaration {
        identity: point.clone(),
        parameters: 0,
        fields: vec![T::Array(Box::new(T::Integer))],
    }])
    .unwrap();
    let key = ErrorInstanceKey {
        arguments: vec![point],
        ..key
    };
    assert!(
        schema
            .instantiate_with_declarations(&key, &mut ErrorInstantiationBudget::default(), &bad)
            .is_err()
    );
}

#[test]
fn generic_fields_are_substituted_and_conflicting_or_missing_declarations_fail() {
    let template = EqualityDeclaration {
        identity: nominal("test.box", vec![T::Parameter(0)]),
        parameters: 1,
        fields: vec![T::Parameter(0)],
    };
    let types = EqualityDeclarations::new([template.clone()]).unwrap();
    assert!(types.supports(&nominal("test.box", vec![T::String])));
    assert!(!types.supports(&nominal("test.box", vec![T::Array(Box::new(T::String))])));
    assert!(!types.supports(&nominal("test.unknown", vec![])));
    let changed = EqualityDeclaration {
        fields: vec![T::Array(Box::new(T::Parameter(0)))],
        ..template.clone()
    };
    assert!(EqualityDeclarations::new([template, changed]).is_err());
}

#[test]
fn nominal_cycles_inspect_all_fields_without_a_recursive_stack_limit() {
    let mut declarations = (0..100)
        .map(|index| EqualityDeclaration {
            identity: nominal(&format!("test.node{index}"), vec![]),
            parameters: 0,
            fields: vec![T::Option(Box::new(nominal(
                &format!("test.node{}", (index + 1) % 100),
                vec![],
            )))],
        })
        .collect::<Vec<_>>();
    let root = nominal("test.node0", vec![]);
    assert!(
        EqualityDeclarations::new(declarations.clone())
            .unwrap()
            .supports(&root)
    );
    declarations[99].fields.push(T::Array(Box::new(T::Integer)));
    assert!(
        !EqualityDeclarations::new(declarations)
            .unwrap()
            .supports(&root)
    );
}

#[test]
fn growing_generic_cycles_are_rejected_before_unbounded_substitution() {
    let declarations = EqualityDeclarations::new([EqualityDeclaration {
        identity: nominal("test.growing", vec![T::Parameter(0)]),
        parameters: 1,
        fields: vec![nominal(
            "test.growing",
            vec![T::Option(Box::new(T::Parameter(0)))],
        )],
    }])
    .unwrap();
    assert!(!declarations.supports(&nominal("test.growing", vec![T::Integer])));
}

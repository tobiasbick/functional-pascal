//! Compiled interfaces retain distinct, canonically qualified parameter identities.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use super::*;

fn nested_interface() -> UnitInterface {
    let mut interface = sample_interface();
    let InterfaceType::Function(outer) = &mut interface.symbols[0].ty else {
        panic!("function fixture");
    };
    let mut inner_parameter = outer.type_parameters[0].clone();
    inner_parameter.identity.offset += 10;
    let outer_parameter = outer.type_parameters[0].clone();
    outer.result = Some(Box::new(InterfaceType::Function(CallableType {
        pure: false,
        type_parameters: vec![inner_parameter.clone()],
        parameters: vec![ParameterType {
            name: "Value".to_owned(),
            mutable: false,
            ty: InterfaceType::GenericParameter(inner_parameter),
        }],
        result: Some(Box::new(InterfaceType::GenericParameter(outer_parameter))),
        variadic: false,
    })));
    interface
}

#[test]
fn generic_parameter_identity_round_trips_enclosing_and_nested_references() {
    let source = nested_interface();
    let bytes = encode_interface(&source).expect("encode nested generics");
    let decoded = decode_interface(&bytes).expect("decode nested generics");
    assert_eq!(decoded, source.canonicalized());
    let InterfaceType::Function(outer) = &decoded
        .symbols
        .iter()
        .find(|symbol| symbol.name == "Transform")
        .expect("Transform symbol")
        .ty
    else {
        panic!("function fixture");
    };
    let InterfaceType::Function(inner) = outer.result.as_deref().expect("outer result") else {
        panic!("inner function fixture");
    };
    let InterfaceType::GenericParameter(result) = inner.result.as_deref().expect("inner result")
    else {
        panic!("outer parameter result");
    };
    assert_eq!(result.identity, outer.type_parameters[0].identity);
    assert_ne!(result.identity, inner.type_parameters[0].identity);
}

#[test]
fn generic_parameter_identity_canonicalizes_units_and_changes_dependency_hashes() {
    let source = nested_interface();
    let mut case = source.clone();
    let InterfaceType::Function(outer) = &mut case.symbols[0].ty else {
        panic!("function fixture");
    };
    outer.type_parameters[0].identity.unit = Some("Demo.Api".to_owned());
    assert_eq!(
        source.digest().expect("source digest"),
        case.digest().expect("case digest")
    );
    let mut changed = source.clone();
    let InterfaceType::Function(outer) = &mut changed.symbols[0].ty else {
        panic!("function fixture");
    };
    let InterfaceType::Function(inner) = outer.result.as_deref_mut().expect("outer result") else {
        panic!("inner function fixture");
    };
    inner.result = Some(Box::new(InterfaceType::GenericParameter(
        inner.type_parameters[0].clone(),
    )));
    assert_ne!(
        source.digest().expect("source digest"),
        changed.digest().expect("changed digest")
    );
}

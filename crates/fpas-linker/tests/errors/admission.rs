//! Concrete artifact admission must enforce payload capabilities without schemas.

use super::fixtures::inputs;
use fpas_linker::link_objects;
use fpas_runtime::{NominalKind, TypeIdentity, ValidationErrorKind};
use fpas_unit::object::{RelocatableObject, decode_object};

#[test]
fn concrete_payloads_cannot_bypass_printable_after_schemas_are_removed() {
    let (libraries, root) = inputs();
    let linked = link_objects(&libraries, &root).expect("typed error fixture");
    let object = RelocatableObject::from_executable("concrete.unit", linked.clone())
        .expect("schema-free concrete object");
    assert!(object.error_schemas.is_empty());
    decode_object(&serde_json::to_vec(&object).expect("valid object wire"))
        .expect("positive concrete object control");
    let rejected = [
        TypeIdentity::Callable {
            parameters: vec![],
            result: None,
            variadic: false,
        },
        TypeIdentity::Nominal {
            kind: NominalKind::Distinct,
            declaration: "example.distinct".to_owned(),
            arguments: vec![],
        },
    ];
    for ty in rejected {
        let mut bad_object = object.clone();
        let leaf = bad_object
            .enums
            .iter_mut()
            .flat_map(|layout| &mut layout.variants)
            .find_map(|variant| {
                variant
                    .error
                    .as_mut()
                    .filter(|error| error.cause.is_none() && !error.field_types.is_empty())
            })
            .expect("leaf payload");
        leaf.field_types[0] = ty.clone();
        let wire = serde_json::to_vec(&bad_object).expect("structurally valid malformed wire");
        let error = decode_object(&wire).expect_err("invalid capability rejected on decode");
        assert!(
            error
                .to_string()
                .contains("error payload requires Printable")
        );

        let mut bad_image = linked.clone().into_unverified();
        let leaf = bad_image
            .enum_variants
            .iter_mut()
            .find_map(|variant| {
                variant
                    .error
                    .as_mut()
                    .filter(|error| error.cause.is_none() && !error.field_types.is_empty())
            })
            .expect("leaf payload");
        leaf.field_types[0] = ty;
        assert!(matches!(
            bad_image
                .verify()
                .expect_err("invalid executable capability")
                .kind,
            ValidationErrorKind::ErrorMetadata {
                reason: "error payload requires Printable"
            }
        ));
    }
}

#[test]
fn source_less_requests_share_one_metadata_expansion_budget() {
    use super::fixtures::{key, request, retained};
    use fpas_unit::errors::{ErrorSchema, ErrorSchemaField, ErrorSchemaVariant};
    use fpas_unit::object::ObjectConstant;
    let (mut libraries, mut root) = inputs();
    libraries[0].error_schemas.push(retained(ErrorSchema {
        declaration: "example.budget".to_owned(),
        display_name: "Example.Budget".to_owned(),
        parameters: vec![None],
        variants: (0..20)
            .map(|variant| ErrorSchemaVariant {
                name: format!("Failure{variant}"),
                message: None,
                cause: None,
                fields: (0..250)
                    .map(|field| ErrorSchemaField {
                        name: format!("Value{field}"),
                        ty: TypeIdentity::Integer,
                    })
                    .collect(),
            })
            .collect(),
    }));
    let add = |root: &mut RelocatableObject, index| {
        root.constants
            .push(ObjectConstant::ErrorConstructor(request(key(
                "example.budget",
                vec![TypeIdentity::Subrange {
                    lower: index,
                    upper: index,
                }],
            ))));
    };
    add(&mut root, 0);
    link_objects(&libraries, &root).expect("bounded positive control");
    for index in 1..201 {
        add(&mut root, index);
    }
    assert!(
        matches!(
            link_objects(&libraries, &root),
            Err(fpas_linker::LinkError::ErrorMetadata(message))
                if message == "error instantiation exceeds the shared metadata budget"
        ),
        "small requests must not expand into unbounded aggregate metadata"
    );
}

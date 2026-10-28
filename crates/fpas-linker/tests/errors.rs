//! Concrete generic identity, source-less artifacts and production construction guards.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "artifact fixtures use explicit boundary assertions"
)]

#[path = "errors/admission.rs"]
mod admission;
mod common;
#[path = "errors/fixtures.rs"]
mod fixtures;

use fixtures::*;
use fpas_linker::{LinkError, link_objects};
use fpas_runtime::execution::CallbackSession;
use fpas_runtime::{DebugType, DebugTypeId, EnumTypeId, StringId, TypeIdentity, Value};
use fpas_unit::object::{decode_object, encode_object};

#[test]
fn two_generic_instances_share_erased_code_and_survive_source_less_round_trips() {
    let (libraries, root) = inputs();
    let libraries = libraries
        .iter()
        .map(|object| {
            decode_object(&encode_object(object).expect("unit object bytes"))
                .expect("unit object admission")
        })
        .collect::<Vec<_>>();
    let root = decode_object(&encode_object(&root).expect("root bytes")).expect("root admission");
    let linked = link_objects(&libraries, &root).expect("source-less schemas and factories");
    let image = linked.executable();
    let instances = image
        .enums
        .iter()
        .filter_map(|layout| layout.error.as_ref())
        .filter(|key| key.declaration == "example.context")
        .collect::<Vec<_>>();
    assert_eq!(instances.len(), 2);
    assert_ne!(instances[0], instances[1]);
    assert_eq!(
        image
            .functions
            .iter()
            .filter(
                |info| image.strings.get(info.name).is_some_and(|name| matches!(
                    name,
                    "Example.Fs.Missing" | "Example.Parse.Missing" | "Example.Context.Failed"
                ))
            )
            .count(),
        4,
        "duplicate requests must share factories"
    );
    assert_eq!(
        image
            .functions
            .iter()
            .filter(|info| image.strings.get(info.name) == Some("library.unit.wrap"))
            .count(),
        1
    );

    let mut stripped = linked.into_unverified();
    for function in &mut stripped.functions {
        function.debug = Default::default();
    }
    stripped.debug_types = vec![DebugType::Dynamic];
    for variant in &mut stripped.enum_variants {
        variant.field_types.fill(DebugTypeId::new(0));
    }
    let paths = stripped
        .source_map
        .sources
        .iter()
        .map(|id| stripped.strings.get(*id).expect("source label").to_owned())
        .collect::<Vec<_>>();
    let hashes = paths
        .iter()
        .map(|path| fpas_program::Digest::of(path.as_bytes()))
        .collect::<Vec<_>>();
    let image = fpas_program::ProgramImage::new(
        fpas_program::ProgramIdentity {
            contracts_enabled: true,
            compiler_version: "error-artifact-fixture".to_owned(),
            bytecode_version: fpas_runtime::BYTECODE_VERSION,
            source_hash: fpas_program::Digest::of(b"fixture"),
            options_hash: fpas_program::Digest::of(b"options"),
            units: vec![],
        },
        paths,
        hashes,
        stripped.verify().expect("stripped executable"),
    )
    .expect("portable program image");
    let bytes = fpas_program::encode(&image).expect("encode stripped program");
    let decoded = fpas_program::decode(&bytes).expect("decode stripped program");
    let executable = decoded.executable().clone();
    let fs = key("example.fs", vec![]);
    let parse = key("example.parse", vec![]);
    let fs_factory = factory(&executable, &request(fs.clone()));
    let parse_factory = factory(&executable, &request(parse.clone()));
    let fs_wrapper = factory(
        &executable,
        &request(key("example.context", vec![fs.as_type()])),
    );
    let parse_wrapper = factory(
        &executable,
        &request(key("example.context", vec![parse.as_type()])),
    );
    let wrapper = function(&executable, "library.unit.wrap");
    let fs_callable = callable(&executable, fs_wrapper);
    let parse_callable = callable(&executable, parse_wrapper);
    let mut session = CallbackSession::new(executable);
    let fs_value = session
        .invoke(fs_factory, vec![Value::Str("settings.toml".into()).into()])
        .expect("Fs leaf")
        .value;
    let parse_value = session
        .invoke(parse_factory, vec![Value::Str("value".into()).into()])
        .expect("Parse leaf")
        .value;
    let fs_error = session
        .invoke(wrapper, vec![fs_value.into(), fs_callable.into()])
        .expect("erased Fs wrapper")
        .value
        .try_to_data()
        .expect("pure error data");
    let parse_error = session
        .invoke(
            wrapper,
            vec![parse_value.clone().into(), parse_callable.into()],
        )
        .expect("erased Parse wrapper")
        .value
        .try_to_data()
        .expect("pure error data");
    assert_eq!(
        fs_error.to_string(),
        "Example.Context.Failed: Operation failed\ncaused by Example.Fs.Missing: Resource missing\n  Path = \"settings.toml\""
    );
    assert_eq!(
        parse_error.to_string(),
        "Example.Context.Failed: Operation failed\ncaused by Example.Parse.Missing: Resource missing\n  Path = \"value\""
    );
    let Value::Enum(fs_error) = fs_error else {
        panic!("typed error");
    };
    let Value::Enum(parse_error) = parse_error else {
        panic!("typed error");
    };
    assert_ne!(
        fs_error.body().layout.enumeration,
        parse_error.body().layout.enumeration
    );
    let failure = session
        .invoke(fs_wrapper, vec![parse_value.into()])
        .expect_err("wrong concrete Cause must fail at construction");
    assert_eq!(
        failure.code,
        fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE
    );
}

#[test]
fn executable_admission_rejects_each_corrupted_error_relationship() {
    let (libraries, root) = inputs();
    let linked = link_objects(&libraries, &root).expect("fixture");
    let image = linked.executable();
    let cause_variant = image
        .enum_variants
        .iter()
        .position(|variant| {
            variant
                .error
                .as_ref()
                .is_some_and(|error| error.cause.is_some())
        })
        .expect("cause variant");
    let leaf_family = image
        .enums
        .iter()
        .position(|layout| {
            layout
                .error
                .as_ref()
                .is_some_and(|key| key.declaration == "example.parse")
        })
        .expect("parse family");
    let cases: &[fn(&mut fpas_runtime::Executable, usize, usize)] = &[
        |image, variant, _| image.enum_variants[variant].error = None,
        |image, variant, _| {
            image.enum_variants[variant]
                .error
                .as_mut()
                .expect("error")
                .message = Some(StringId::new(u32::MAX))
        },
        |image, variant, _| {
            image.enum_variants[variant]
                .error
                .as_mut()
                .expect("error")
                .cause
                .as_mut()
                .expect("cause")
                .field = u16::MAX
        },
        |image, variant, other| {
            let error = image.enum_variants[variant].error.as_mut().expect("error");
            let current = usize::from(error.cause.expect("cause").family.get());
            let replacement = if current == other {
                image
                    .enums
                    .iter()
                    .position(|layout| {
                        layout
                            .error
                            .as_ref()
                            .is_some_and(|key| key.declaration == "example.fs")
                    })
                    .expect("Fs")
            } else {
                other
            };
            error.cause.as_mut().expect("cause").family =
                EnumTypeId::try_from_index(replacement).expect("family ID");
        },
        |image, variant, _| {
            image.enum_variants[variant]
                .error
                .as_mut()
                .expect("error")
                .field_types
                .clear()
        },
        |image, variant, _| {
            let owner = image.enum_variants[variant].owner;
            image.enums[usize::from(owner.get())]
                .error
                .as_mut()
                .expect("key")
                .arguments = vec![TypeIdentity::Parameter(0)];
        },
    ];
    for mutate in cases {
        let mut bad = linked.clone().into_unverified();
        mutate(&mut bad, cause_variant, leaf_family);
        assert!(matches!(
            bad.verify().expect_err("malformed error metadata").kind,
            fpas_runtime::ValidationErrorKind::ErrorMetadata { .. }
        ));
    }
}

#[test]
fn schema_constraints_and_requests_fail_before_execution() {
    let (libraries, root) = inputs();
    let mut wrong_bound = root.clone();
    wrong_bound
        .constants
        .push(fpas_unit::object::ObjectConstant::ErrorConstructor(
            request(key("example.context", vec![TypeIdentity::Integer])),
        ));
    assert!(matches!(
        link_objects(&libraries, &wrong_bound),
        Err(LinkError::ErrorMetadata(_))
    ));
    let mut missing = root.clone();
    missing
        .constants
        .push(fpas_unit::object::ObjectConstant::ErrorConstructor(
            request(key("example.missing", vec![])),
        ));
    assert!(matches!(
        link_objects(&libraries, &missing),
        Err(LinkError::ErrorMetadata(_))
    ));
    let mut conflict = libraries[0].clone();
    conflict.owner = "other.unit".to_owned();
    conflict.functions.clear();
    conflict.definitions.clear();
    conflict.error_schemas[0].schema.variants[0].message =
        Some("Conflicting declaration".to_owned());
    let mut libraries = libraries;
    libraries.push(conflict);
    assert!(matches!(
        link_objects(&libraries, &root),
        Err(LinkError::ErrorMetadata(_))
    ));
}

#[test]
fn nested_causes_and_non_cause_arguments_keep_full_semantic_identity() {
    use fpas_unit::{
        errors::{ErrorSchemaField, ErrorSchemaVariant},
        interface::TypeConstraint,
        object::ObjectConstant,
    };
    let (mut libraries, mut root) = inputs();
    let mut data = context();
    data.declaration = "example.data".to_owned();
    data.display_name = "Example.Data".to_owned();
    data.parameters = vec![Some(TypeConstraint::Printable)];
    data.variants = vec![ErrorSchemaVariant {
        name: "Value".to_owned(),
        message: None,
        cause: None,
        fields: vec![ErrorSchemaField {
            name: "Value".to_owned(),
            ty: TypeIdentity::Parameter(0),
        }],
    }];
    libraries[0].error_schemas.push(retained(data));
    for ty in [
        TypeIdentity::Integer,
        TypeIdentity::String,
        ordinary_nominal_argument("example.record"),
        fpas_runtime::TypeIdentity::Nominal {
            kind: fpas_runtime::NominalKind::Record,
            declaration: "example.record".to_owned(),
            arguments: vec![TypeIdentity::Integer],
        },
        fpas_runtime::TypeIdentity::Nominal {
            kind: fpas_runtime::NominalKind::Record,
            declaration: "example.record".to_owned(),
            arguments: vec![TypeIdentity::String],
        },
    ] {
        root.constants
            .push(ObjectConstant::ErrorConstructor(request(key(
                "example.data",
                vec![ty],
            ))));
    }
    let nested = key(
        "example.context",
        vec![key("example.context", vec![key("example.fs", vec![]).as_type()]).as_type()],
    );
    root.constants
        .push(ObjectConstant::ErrorConstructor(request(nested.clone())));
    let image = link_objects(&libraries, &root).expect("nested and non-cause instances");
    let image = image.executable();
    assert_eq!(
        image
            .enums
            .iter()
            .filter(|layout| layout
                .error
                .as_ref()
                .is_some_and(|key| key.declaration == "example.data"))
            .count(),
        5
    );
    let owner = image
        .enums
        .iter()
        .position(|layout| layout.error.as_ref() == Some(&nested))
        .expect("nested instance");
    let cause = image
        .enum_variants
        .iter()
        .find(|variant| usize::from(variant.owner.get()) == owner)
        .expect("nested variant")
        .error
        .as_ref()
        .expect("error")
        .cause
        .expect("cause");
    assert_eq!(
        image.enums[usize::from(cause.family.get())].error.as_ref(),
        Some(&key(
            "example.context",
            vec![key("example.fs", vec![]).as_type()]
        ))
    );
}

#[test]
fn stable_recursive_schemas_link_but_expanding_instance_requests_are_bounded() {
    use fpas_unit::{errors::ErrorSchemaVariant, object::ObjectConstant};
    let (mut libraries, mut root) = inputs();
    let mut recursive = context();
    recursive.declaration = "example.recursive".to_owned();
    recursive.display_name = "Example.Recursive".to_owned();
    recursive.variants[0].fields[0].ty =
        key("example.recursive", vec![TypeIdentity::Parameter(0)]).as_type();
    recursive.variants.push(ErrorSchemaVariant {
        name: "End".to_owned(),
        fields: vec![],
        message: None,
        cause: None,
    });
    libraries[0].error_schemas.push(retained(recursive.clone()));
    root.constants
        .push(ObjectConstant::ErrorConstructor(request(key(
            "example.recursive",
            vec![key("example.fs", vec![]).as_type()],
        ))));
    link_objects(&libraries, &root).expect("finite recursion reserves one instance key");
    let schema = libraries[0]
        .error_schemas
        .last_mut()
        .expect("recursive schema");
    schema.schema.variants[0].fields[0].ty = key(
        "example.recursive",
        vec![key("example.context", vec![TypeIdentity::Parameter(0)]).as_type()],
    )
    .as_type();
    assert!(matches!(
        link_objects(&libraries, &root),
        Err(LinkError::ErrorMetadata(_))
    ));
}

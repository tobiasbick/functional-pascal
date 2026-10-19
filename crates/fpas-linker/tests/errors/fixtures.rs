//! Shared public-boundary schema, object and callback fixtures.

use fpas_bytecode::{
    ErrorInstanceKey, FunctionId, Instruction, NominalKind, Opcode, TypeIdentity, Value,
    VerifiedExecutable,
};
use fpas_unit::{
    errors::{
        ErrorConstructorRequest, ErrorConstructorSignature, ErrorSchema, ErrorSchemaField,
        ErrorSchemaVariant,
    },
    interface::TypeConstraint,
    object::{ObjectConstant, ObjectFunction, ObjectReturn, RelocatableObject},
};

pub fn key(name: &str, arguments: Vec<TypeIdentity>) -> ErrorInstanceKey {
    ErrorInstanceKey {
        declaration: name.to_owned(),
        arguments,
    }
}

pub fn request(instance: ErrorInstanceKey) -> ErrorConstructorRequest {
    ErrorConstructorRequest {
        instance,
        variant: 0,
    }
}

pub fn leaf(name: &str, display: &str) -> ErrorSchema {
    ErrorSchema {
        declaration: name.to_owned(),
        display_name: display.to_owned(),
        parameters: vec![],
        variants: vec![ErrorSchemaVariant {
            name: "Missing".to_owned(),
            message: Some("Resource missing".to_owned()),
            cause: None,
            fields: vec![ErrorSchemaField {
                name: "Path".to_owned(),
                ty: TypeIdentity::String,
            }],
        }],
    }
}

pub fn context() -> ErrorSchema {
    ErrorSchema {
        declaration: "example.context".to_owned(),
        display_name: "Example.Context".to_owned(),
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
    }
}

pub fn inputs() -> (Vec<RelocatableObject>, RelocatableObject) {
    let mut library = crate::common::unit(true);
    library.error_schemas = vec![
        leaf("example.fs", "Example.Fs"),
        leaf("example.parse", "Example.Parse"),
        context(),
    ]
    .into_iter()
    .map(retained)
    .collect();
    let fs = key("example.fs", vec![]);
    let parse = key("example.parse", vec![]);
    library
        .constants
        .push(ObjectConstant::ErrorConstructor(request(key(
            "example.context",
            vec![fs.as_type()],
        ))));
    library.functions.push(ObjectFunction {
        error_constructors: Some(ErrorConstructorSignature {
            type_parameters: 1,
            requirements: vec![request(key(
                "example.context",
                vec![TypeIdentity::Parameter(0)],
            ))],
        }),
        name: "library.unit.wrap".to_owned(),
        code: vec![
            Instruction::abc(Opcode::CallValue, 2, 1, 0, 1)
                .expect("generic factory call")
                .word(),
            Instruction::abc(Opcode::Return, 2, 0, 0, 0)
                .expect("return wrapper")
                .word(),
        ],
        arity: 2,
        capture_count: 0,
        register_count: 3,
        returns: ObjectReturn::Value,
        uses_spawn_tasks: false,
        source_runs: vec![fpas_unit::object::ObjectSourceRun {
            instruction_start: 0,
            source: 0,
            line: 1,
            column: 1,
        }],
        debug: Default::default(),
    });
    library.define_all_private().expect("library definitions");
    let mut root = crate::common::program();
    root.imports.clear();
    root.relocations.clear();
    root.functions[0].code = vec![crate::common::return_unit()];
    root.constants = vec![
        ObjectConstant::ErrorConstructor(request(fs.clone())),
        ObjectConstant::ErrorConstructor(request(parse.clone())),
        ObjectConstant::ErrorConstructor(request(key("example.context", vec![fs.as_type()]))),
        ObjectConstant::ErrorConstructor(request(key("example.context", vec![parse.as_type()]))),
    ];
    (vec![library], root)
}

pub fn factory(executable: &VerifiedExecutable, request: &ErrorConstructorRequest) -> FunctionId {
    function(
        executable,
        &fpas_unit::errors::constructor_symbol(request).expect("factory name"),
    )
}

pub fn function(executable: &VerifiedExecutable, name: &str) -> FunctionId {
    FunctionId::try_from_index(
        executable
            .executable()
            .functions
            .iter()
            .position(|function| executable.executable().strings.get(function.name) == Some(name))
            .expect("function exists"),
    )
    .expect("function ID")
}

pub fn callable(executable: &VerifiedExecutable, id: FunctionId) -> Value {
    let info = &executable.executable().functions[usize::from(id.get())];
    Value::Function(fpas_bytecode::SharedFunction::unbound(
        id,
        executable
            .executable()
            .strings
            .get(info.name)
            .expect("name"),
        vec![],
    ))
}

pub fn ordinary_nominal_argument(name: &str) -> TypeIdentity {
    TypeIdentity::Nominal {
        kind: NominalKind::Record,
        declaration: name.to_owned(),
        arguments: vec![],
    }
}

pub fn retained(schema: ErrorSchema) -> fpas_unit::object::ObjectErrorSchema {
    fpas_unit::object::ObjectErrorSchema {
        schema,
        location: fpas_unit::object::ObjectDebugLocation {
            source: 0,
            line: 1,
            column: 1,
        },
    }
}

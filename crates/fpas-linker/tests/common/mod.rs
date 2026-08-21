//! Shared relocatable-object fixtures for linker integration tests.

#![allow(
    clippy::expect_used,
    dead_code,
    reason = "focused linker fixtures use explicit expectations and are shared across test binaries"
)]

use fpas_bytecode::{Instruction, NO_REGISTER, Opcode};
use fpas_unit::object::{
    DefinitionTarget, ImportShape, OBJECT_VERSION, ObjectDefinition, ObjectFunction, ObjectImport,
    ObjectReturn, ObjectSourceRun, RelocatableObject, Relocation, RelocationKind, SymbolReference,
};

pub fn return_unit() -> u64 {
    Instruction::abc(Opcode::Return, NO_REGISTER, 0, 0, 0)
        .expect("return")
        .word()
}

pub fn function(name: &str) -> ObjectFunction {
    ObjectFunction {
        name: name.to_string(),
        code: vec![return_unit()],
        arity: 0,
        capture_count: 0,
        register_count: 0,
        returns: ObjectReturn::Unit,
        uses_spawn_tasks: false,
        source_runs: vec![ObjectSourceRun {
            instruction_start: 0,
            source: 0,
            line: 1,
            column: 1,
        }],
        debug: fpas_unit::object::ObjectFunctionDebugInfo::default(),
    }
}

pub fn unit(public: bool) -> RelocatableObject {
    RelocatableObject {
        version: OBJECT_VERSION,
        owner: "library.unit".to_string(),
        entry: None,
        initializer: None,
        functions: vec![function("library.unit.zed"), function("library.unit.alpha")],
        constants: vec![
            fpas_unit::object::ObjectConstant::Integer(7),
            fpas_unit::object::ObjectConstant::Real(0.0_f64.to_bits()),
            fpas_unit::object::ObjectConstant::Real((-0.0_f64).to_bits()),
        ],
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        debug_types: vec![fpas_unit::object::ObjectDebugType::Dynamic],
        sources: vec!["library.fpas".to_string()],
        definitions: vec![
            ObjectDefinition {
                name: "library.unit.alpha".to_string(),
                target: DefinitionTarget::Function(1),
                public,
            },
            ObjectDefinition {
                name: "library.unit.zed".to_string(),
                target: DefinitionTarget::Function(0),
                public,
            },
        ],
        imports: Vec::new(),
        relocations: Vec::new(),
    }
}

pub fn program() -> RelocatableObject {
    let call = Instruction::abc(Opcode::CallDirect, NO_REGISTER, 0, 0, 0)
        .expect("call")
        .word();
    RelocatableObject {
        version: OBJECT_VERSION,
        owner: "demo".to_string(),
        entry: Some(0),
        initializer: None,
        functions: vec![ObjectFunction {
            name: "demo".to_string(),
            code: vec![call, return_unit()],
            arity: 0,
            capture_count: 0,
            register_count: 0,
            returns: ObjectReturn::Unit,
            uses_spawn_tasks: false,
            source_runs: vec![ObjectSourceRun {
                instruction_start: 0,
                source: 0,
                line: 1,
                column: 1,
            }],
            debug: fpas_unit::object::ObjectFunctionDebugInfo::default(),
        }],
        constants: vec![
            fpas_unit::object::ObjectConstant::Integer(7),
            fpas_unit::object::ObjectConstant::Real((-0.0_f64).to_bits()),
        ],
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        debug_types: vec![fpas_unit::object::ObjectDebugType::Dynamic],
        sources: vec!["program.fpas".to_string()],
        definitions: vec![ObjectDefinition {
            name: "demo".to_string(),
            target: DefinitionTarget::Function(0),
            public: false,
        }],
        imports: vec![ObjectImport {
            name: "library.unit.zed".to_string(),
            shape: ImportShape::Function {
                arity: 0,
                capture_count: 0,
                returns_value: false,
            },
        }],
        relocations: vec![Relocation {
            function: 0,
            instruction: 0,
            kind: RelocationKind::Function(SymbolReference::Import(0)),
        }],
    }
}

pub fn private_record_copy(
    owner: &str,
    debug_types: Vec<fpas_unit::object::ObjectDebugType>,
    layout: fpas_unit::object::ObjectRecordLayout,
) -> RelocatableObject {
    let mut object = unit(owner == "library.unit");
    if owner != "library.unit" {
        for function in &mut object.functions {
            function.name = function.name.replacen("library.unit", owner, 1);
        }
        for definition in &mut object.definitions {
            definition.name = definition.name.replacen("library.unit", owner, 1);
        }
    }
    object.owner = owner.to_string();
    object.debug_types = debug_types;
    let name = layout.name.clone();
    object.records.push(layout);
    object.definitions.push(ObjectDefinition {
        name,
        target: DefinitionTarget::Record(0),
        public: false,
    });
    object
        .definitions
        .sort_by(|left, right| left.name.cmp(&right.name));
    object
}

pub fn one_field_record(field_type: u32) -> fpas_unit::object::ObjectRecordLayout {
    fpas_unit::object::ObjectRecordLayout {
        name: "shared.node".to_string(),
        fields: vec!["value".to_string()],
        field_types: vec![field_type],
        properties: Vec::new(),
        methods: Vec::new(),
    }
}

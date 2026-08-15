//! P8 register-object linking, validation, determinism, and VM integration.

#![allow(
    clippy::expect_used,
    reason = "focused linker fixtures use explicit expectations"
)]

use fpas_bytecode::{Instruction, NO_REGISTER, Opcode, Value};
use fpas_linker::{LinkError, link_objects};
use fpas_unit::object::{
    DefinitionTarget, ImportShape, OBJECT_VERSION, ObjectDefinition, ObjectFunction, ObjectImport,
    ObjectReturn, ObjectSourceRun, RelocatableObject, Relocation, RelocationKind, SymbolReference,
};
use fpas_vm::Vm;

fn return_unit() -> u64 {
    Instruction::abc(Opcode::Return, NO_REGISTER, 0, 0, 0)
        .expect("return")
        .word()
}

fn function(name: &str) -> ObjectFunction {
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

fn unit(public: bool) -> RelocatableObject {
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

fn program() -> RelocatableObject {
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

#[test]
fn dependency_objects_link_deterministically_and_run_in_the_register_vm() {
    let mut unit = unit(true);
    unit.functions[0].debug = fpas_unit::object::ObjectFunctionDebugInfo {
        scopes: vec![fpas_unit::object::ObjectDebugScope {
            id: 0,
            parent: None,
        }],
        bindings: Vec::new(),
        sequence_points: vec![fpas_unit::object::ObjectSequencePoint {
            instruction_start: 0,
            location: fpas_unit::object::ObjectDebugLocation {
                source: 0,
                line: 1,
                column: 1,
            },
            scope: 0,
        }],
        ..Default::default()
    };
    let first =
        link_objects(std::slice::from_ref(&unit), &program()).expect("first deterministic link");
    let second = link_objects(&[unit], &program()).expect("second deterministic link");
    assert_eq!(first.executable(), second.executable());
    assert_eq!(first.executable().functions[0].code.start.get(), 0);
    assert_eq!(
        first
            .executable()
            .strings
            .get(first.executable().functions[1].name),
        Some("library.unit.alpha")
    );
    let call = first.executable().code[0]
        .abc_operands()
        .expect("linked call");
    assert_eq!(
        call.b, 2,
        "canonical unit function order must assign zed after alpha"
    );
    assert_eq!(
        first.executable().constants.len(),
        3,
        "equal constants merge while signed zero remains bit-distinct"
    );
    assert_eq!(
        first.executable().source_map.runs[1]
            .instruction_start
            .get(),
        2,
        "unit source runs are rebased after the root function"
    );
    assert_eq!(
        first.executable().functions[2].debug.sequence_points[0]
            .instruction
            .get(),
        first.executable().functions[2].code.start.get(),
        "unit debugger sequence points are rebased with linked code"
    );
    let execution = Vm::new(first).run().expect("linked VM execution");
    assert_eq!(execution.value, Value::Unit);
}

#[test]
fn linker_retains_result_only_debug_types() {
    let mut library = unit(true);
    library.debug_types = vec![
        fpas_unit::object::ObjectDebugType::Dynamic,
        fpas_unit::object::ObjectDebugType::Integer,
    ];
    library.functions[0].debug.result_type = Some(1);
    let linked =
        link_objects(std::slice::from_ref(&library), &program()).expect("result type link");
    let image = linked.executable();
    let function = image
        .functions
        .iter()
        .find(|function| image.strings.get(function.name) == Some("library.unit.zed"))
        .expect("library function");
    let result_type = function.debug.result_type.expect("portable result type");
    assert_eq!(
        image.debug_types.get(result_type.get() as usize),
        Some(&fpas_bytecode::DebugType::Integer)
    );
}

#[test]
fn linker_relocates_lexical_owner_function_ids() {
    let mut library = unit(true);
    library.functions[0].register_count = 1;
    library.functions[0].debug = fpas_unit::object::ObjectFunctionDebugInfo {
        scopes: vec![fpas_unit::object::ObjectDebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![fpas_unit::object::ObjectDebugBinding {
            name: "captured".to_string(),
            type_name: "dynamic".to_string(),
            ty: 0,
            register: 0,
            kind: fpas_unit::object::ObjectDebugBindingKind::Local,
            mutable: false,
            scope: 0,
            declaration: None,
            hidden: false,
            cell_backed: false,
        }],
        ..Default::default()
    };
    library.functions[1].capture_count = 1;
    library.functions[1].register_count = 1;
    library.functions[1].debug = fpas_unit::object::ObjectFunctionDebugInfo {
        lexical_owner: Some(0),
        capture_sources: vec![fpas_unit::object::ObjectCaptureSource {
            binding: 0,
            ty: 0,
            kind: fpas_unit::object::ObjectCaptureKind::Value,
        }],
        ..Default::default()
    };
    let linked =
        link_objects(std::slice::from_ref(&library), &program()).expect("capture provenance link");
    let image = linked.executable();
    let alpha = image
        .functions
        .iter()
        .find(|function| image.strings.get(function.name) == Some("library.unit.alpha"))
        .expect("alpha");
    let zed_id = image
        .functions
        .iter()
        .position(|function| image.strings.get(function.name) == Some("library.unit.zed"))
        .expect("zed");
    assert_eq!(alpha.capture_count, 1);
    assert_eq!(
        alpha
            .debug
            .lexical_owner
            .map(|owner| usize::from(owner.get())),
        Some(zed_id)
    );
    assert_eq!(alpha.debug.capture_sources.len(), 1);
    assert_eq!(
        alpha.debug.capture_sources[0].kind,
        fpas_bytecode::DebugCaptureKind::Value
    );
}

#[test]
fn unit_initializers_are_prefixed_in_dependency_order_and_require_procedure_abi() {
    let mut library = unit(true);
    library.initializer = Some(0);
    let linked =
        link_objects(std::slice::from_ref(&library), &program()).expect("unit initializer link");
    let first = linked.executable().code[0]
        .abc_operands()
        .expect("initializer call operands");
    let program_call = linked.executable().code[1]
        .abc_operands()
        .expect("program call operands");
    assert_eq!(first.b, 2, "unit initializer must target canonical zed ID");
    assert_eq!(
        program_call.b, 2,
        "root body must follow initializer prefix"
    );
    assert_eq!(
        linked.executable().source_map.runs[0]
            .instruction_start
            .get(),
        0,
        "the synthetic initializer prefix must retain a root source location"
    );

    library.functions[0].arity = 1;
    assert!(matches!(
        link_objects(&[library], &program()),
        Err(LinkError::InvalidInitializer {
            detail: "expected zero parameters",
            ..
        })
    ));
}

#[test]
fn private_missing_wrong_kind_and_incompatible_callable_imports_are_rejected() {
    assert!(matches!(
        link_objects(&[unit(false)], &program()),
        Err(LinkError::PrivateImport { .. })
    ));

    let mut missing = program();
    missing.imports[0].name = "library.unit.missing".to_string();
    assert!(matches!(
        link_objects(&[unit(true)], &missing),
        Err(LinkError::UnresolvedImport { .. })
    ));

    let mut wrong_kind = program();
    wrong_kind.imports[0].shape = ImportShape::Global { mutable: false };
    let wrong_kind_result = link_objects(&[unit(true)], &wrong_kind);
    assert!(
        matches!(wrong_kind_result, Err(LinkError::InvalidObject { .. })),
        "wrong-kind import must fail object validation: {wrong_kind_result:?}"
    );

    let mut wrong_arity = program();
    wrong_arity.imports[0].shape = ImportShape::Function {
        arity: 1,
        capture_count: 0,
        returns_value: false,
    };
    assert!(matches!(
        link_objects(&[unit(true)], &wrong_arity),
        Err(LinkError::IncompatibleImport { .. })
    ));
}

#[test]
fn duplicate_definitions_are_case_insensitive_through_canonical_object_validation() {
    let first = unit(true);
    let mut second = unit(true);
    second.owner = "other.unit".to_string();
    assert!(matches!(
        link_objects(&[first, second], &program()),
        Err(LinkError::DuplicateDefinition(name)) if name == "library.unit.alpha"
    ));
}

#[test]
fn matching_private_layout_copies_share_one_canonical_type_id() {
    let mut first = unit(true);
    first.records.push(fpas_unit::object::ObjectRecordLayout {
        name: "std.console.keyevent".to_string(),
        fields: vec!["kind".to_string(), "character".to_string()],
        field_types: vec![0, 0],
        properties: Vec::new(),
        methods: Vec::new(),
    });
    first.definitions.push(ObjectDefinition {
        name: "std.console.keyevent".to_string(),
        target: DefinitionTarget::Record(0),
        public: false,
    });
    first
        .definitions
        .sort_by(|left, right| left.name.cmp(&right.name));

    let mut second = unit(false);
    second.owner = "other.unit".to_string();
    for function in &mut second.functions {
        function.name = function.name.replacen("library.unit", "other.unit", 1);
    }
    for definition in &mut second.definitions {
        definition.name = definition.name.replacen("library.unit", "other.unit", 1);
    }
    second.records.push(fpas_unit::object::ObjectRecordLayout {
        name: "std.console.keyevent".to_string(),
        fields: vec!["kind".to_string(), "character".to_string()],
        field_types: vec![0, 0],
        properties: Vec::new(),
        methods: Vec::new(),
    });
    second.definitions.push(ObjectDefinition {
        name: "std.console.keyevent".to_string(),
        target: DefinitionTarget::Record(0),
        public: false,
    });
    second
        .definitions
        .sort_by(|left, right| left.name.cmp(&right.name));

    let executable = link_objects(&[first, second], &program())
        .expect("matching private layout copies must coalesce");
    assert_eq!(executable.executable().records.len(), 1);
}

#[test]
fn incompatible_record_layout_import_is_rejected_before_relocation() {
    let mut library = unit(true);
    library.records.push(fpas_unit::object::ObjectRecordLayout {
        name: "library.unit.point".to_string(),
        fields: vec!["x".to_string(), "y".to_string()],
        field_types: vec![0, 0],
        properties: Vec::new(),
        methods: Vec::new(),
    });
    library.definitions.push(ObjectDefinition {
        name: "library.unit.point".to_string(),
        target: DefinitionTarget::Record(0),
        public: true,
    });
    library
        .definitions
        .sort_by(|left, right| left.name.cmp(&right.name));
    let mut root = program();
    root.imports.push(ObjectImport {
        name: "library.unit.point".to_string(),
        shape: ImportShape::Record {
            fields: vec!["y".to_string(), "x".to_string()],
        },
    });
    root.imports
        .sort_by(|left, right| left.name.cmp(&right.name));
    let function_import = root
        .imports
        .iter()
        .position(|import| import.name == "library.unit.zed")
        .expect("function import index");
    root.relocations[0].kind = RelocationKind::Function(SymbolReference::Import(
        u32::try_from(function_import).expect("import index"),
    ));
    assert!(matches!(
        link_objects(&[library], &root),
        Err(LinkError::IncompatibleImport { .. })
    ));
}

#[test]
fn imported_global_record_and_enum_references_become_dense_numeric_ids() {
    let mut library = unit(true);
    library.globals.push(fpas_unit::object::ObjectGlobal {
        name: "library.unit.counter".to_string(),
        ty: 0,
        mutable: true,
    });
    library.records.push(fpas_unit::object::ObjectRecordLayout {
        name: "library.unit.point".to_string(),
        fields: vec!["x".to_string()],
        field_types: vec![0],
        properties: Vec::new(),
        methods: vec![fpas_unit::object::ObjectRecordMethod {
            name: "translate".to_string(),
            routine: "library.unit.alpha".to_string(),
        }],
    });
    library.enums.push(fpas_unit::object::ObjectEnumLayout {
        name: "library.unit.choice".to_string(),
        variants: vec![fpas_unit::object::ObjectEnumVariant {
            name: "some".to_string(),
            fields: vec!["value".to_string()],
            field_types: vec![0],
        }],
    });
    library.debug_types = vec![
        fpas_unit::object::ObjectDebugType::Integer,
        fpas_unit::object::ObjectDebugType::Record("library.unit.point".to_string()),
        fpas_unit::object::ObjectDebugType::Enum("library.unit.choice".to_string()),
        fpas_unit::object::ObjectDebugType::Array(0),
    ];
    library.definitions.extend([
        ObjectDefinition {
            name: "library.unit.choice".to_string(),
            target: DefinitionTarget::Enum(0),
            public: true,
        },
        ObjectDefinition {
            name: "library.unit.counter".to_string(),
            target: DefinitionTarget::Global(0),
            public: true,
        },
        ObjectDefinition {
            name: "library.unit.point".to_string(),
            target: DefinitionTarget::Record(0),
            public: true,
        },
    ]);
    library
        .definitions
        .sort_by(|left, right| left.name.cmp(&right.name));

    let mut root = program();
    root.functions[0].code = vec![
        Instruction::abx(Opcode::LoadGlobal, 0, 0)
            .expect("global")
            .word(),
        Instruction::abc(Opcode::StoreGlobalIndexPath, 0, 0, 1, 0)
            .expect("global index path")
            .word(),
        Instruction::abc(Opcode::MakeRecord, 1, 0, 0, 0)
            .expect("record")
            .word(),
        Instruction::abc(Opcode::MakeEnum, 2, 0, 0, 0)
            .expect("enum")
            .word(),
        return_unit(),
    ];
    root.functions[0].register_count = 3;
    root.imports = vec![
        ObjectImport {
            name: "library.unit.choice".to_string(),
            shape: ImportShape::Enum {
                variants: vec![("some".to_string(), vec!["value".to_string()])],
            },
        },
        ObjectImport {
            name: "library.unit.counter".to_string(),
            shape: ImportShape::Global { mutable: true },
        },
        ObjectImport {
            name: "library.unit.point".to_string(),
            shape: ImportShape::Record {
                fields: vec!["x".to_string()],
            },
        },
    ];
    root.relocations = vec![
        Relocation {
            function: 0,
            instruction: 0,
            kind: RelocationKind::Global(SymbolReference::Import(1)),
        },
        Relocation {
            function: 0,
            instruction: 1,
            kind: RelocationKind::Global(SymbolReference::Import(1)),
        },
        Relocation {
            function: 0,
            instruction: 2,
            kind: RelocationKind::Record(SymbolReference::Import(2)),
        },
        Relocation {
            function: 0,
            instruction: 3,
            kind: RelocationKind::EnumVariant {
                enumeration: SymbolReference::Import(0),
                variant: "some".to_string(),
            },
        },
    ];
    let linked = link_objects(&[library], &root).expect("layout link");
    assert_eq!(linked.executable().globals.len(), 1);
    assert_eq!(linked.executable().records.len(), 1);
    let method = linked.executable().records[0].methods[0];
    assert_eq!(
        linked.executable().strings.get(method.name),
        Some("translate")
    );
    assert_eq!(
        linked.executable().strings.get(method.routine),
        Some("library.unit.alpha")
    );
    assert_eq!(linked.executable().enums.len(), 1);
    assert_eq!(linked.executable().enum_variants.len(), 1);
    assert_eq!(
        linked.executable().debug_types[..4],
        [
            fpas_bytecode::DebugType::Integer,
            fpas_bytecode::DebugType::Record(fpas_bytecode::RecordTypeId::new(0)),
            fpas_bytecode::DebugType::Enum(fpas_bytecode::EnumTypeId::new(0)),
            fpas_bytecode::DebugType::Array(fpas_bytecode::DebugTypeId::new(0)),
        ]
    );
    assert_eq!(linked.executable().globals[0].ty.get(), 0);
    assert_eq!(linked.executable().records[0].fields[0].ty.get(), 0);
    assert_eq!(linked.executable().enum_variants[0].field_types[0].get(), 0);
    assert_eq!(
        linked.executable().code[0]
            .abx_operands()
            .expect("global operands")
            .bx,
        0
    );
    assert_eq!(
        linked.executable().code[1]
            .abc_operands()
            .expect("global index path operands")
            .b,
        0
    );
    assert_eq!(
        linked.executable().code[2]
            .abc_operands()
            .expect("record operands")
            .b,
        0
    );
    assert_eq!(
        linked.executable().code[3]
            .abc_operands()
            .expect("enum operands")
            .b,
        0
    );
}

#[test]
fn unit_entry_and_function_id_overflow_fail_before_executable_creation() {
    let mut entry_unit = unit(true);
    entry_unit.entry = Some(0);
    assert!(matches!(
        link_objects(&[entry_unit], &program()),
        Err(LinkError::UnitEntry(_))
    ));

    let mut oversized = unit(true);
    oversized.functions = (0..=u16::MAX)
        .map(|index| function(&format!("library.unit.f{index:05}")))
        .chain(std::iter::once(function("library.unit.overflow")))
        .collect();
    oversized.definitions.clear();
    let mut root = program();
    root.functions[0].code = vec![return_unit()];
    root.imports.clear();
    root.relocations.clear();
    let overflow = link_objects(&[oversized], &root);
    assert!(
        matches!(overflow, Err(LinkError::Overflow("function IDs"))),
        "unexpected overflow result: {overflow:?}"
    );
}

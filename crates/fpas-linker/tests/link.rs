//! P8 register-object linking, validation, determinism, and VM integration.

#![allow(
    clippy::expect_used,
    reason = "focused linker fixtures use explicit expectations"
)]

mod common;

use fpas_bytecode::{Instruction, Opcode, Value};
use fpas_linker::{LinkError, link_objects};
use fpas_unit::object::{
    DefinitionTarget, ImportShape, ObjectDefinition, Relocation, RelocationKind, SymbolReference,
};
use fpas_vm::Vm;

use common::*;

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
fn linker_rebases_exact_local_and_global_initializer_stores() {
    let mut library = unit(true);
    library.functions[0].register_count = 1;
    library.functions[0].code = vec![
        Instruction::abx(Opcode::LoadConstant, 0, 0)
            .expect("initializer value")
            .word(),
        Instruction::abx(Opcode::StoreGlobal, 0, 0)
            .expect("global initializer")
            .word(),
        Instruction::abc(Opcode::Move, 0, 0, 0, 0)
            .expect("local initializer")
            .word(),
        return_unit(),
    ];
    library.functions[0].debug = fpas_unit::object::ObjectFunctionDebugInfo {
        scopes: vec![fpas_unit::object::ObjectDebugScope {
            id: 0,
            parent: None,
        }],
        bindings: vec![fpas_unit::object::ObjectDebugBinding {
            name: "value".to_string(),
            type_name: "dynamic".to_string(),
            ty: 0,
            register: 0,
            kind: fpas_unit::object::ObjectDebugBindingKind::Local,
            mutable: true,
            scope: 0,
            declaration: None,
            hidden: false,
            cell_backed: false,
            initializer_start: Some(2),
        }],
        ..Default::default()
    };
    library.globals.push(fpas_unit::object::ObjectGlobal {
        name: "library.unit.counter".to_string(),
        ty: 0,
        mutable: true,
        initializer: Some(fpas_unit::object::ObjectInitializer {
            function: 0,
            instruction_start: 1,
        }),
    });
    library.definitions.push(ObjectDefinition {
        name: "library.unit.counter".to_string(),
        target: DefinitionTarget::Global(0),
        public: false,
    });
    library
        .definitions
        .sort_by(|left, right| left.name.cmp(&right.name));
    library.relocations.extend([
        Relocation {
            function: 0,
            instruction: 0,
            kind: RelocationKind::Constant(0),
        },
        Relocation {
            function: 0,
            instruction: 1,
            kind: RelocationKind::Global(SymbolReference::Local(0)),
        },
    ]);

    let linked = link_objects(std::slice::from_ref(&library), &program())
        .expect("initializer metadata link");
    let image = linked.executable();
    let function = image
        .functions
        .iter()
        .find(|function| image.strings.get(function.name) == Some("library.unit.zed"))
        .expect("linked initializer owner");
    assert_eq!(
        function.debug.bindings[0].initializer,
        Some(fpas_bytecode::InstructionAddress::new(
            function.code.start.get() + 2
        ))
    );
    let global = image
        .globals
        .iter()
        .find(|global| image.strings.get(global.name) == Some("library.unit.counter"))
        .expect("linked initialized global");
    let initializer = global.initializer.expect("linked global initializer");
    assert_eq!(initializer.function, fpas_bytecode::FunctionId::new(2));
    assert_eq!(initializer.instruction.get(), function.code.start.get() + 1);
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
            initializer_start: None,
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

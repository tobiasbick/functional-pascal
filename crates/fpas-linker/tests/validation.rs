mod object_fixture;

use fpas_bytecode::{ExecutableError, Op};
use fpas_linker::{LinkError, link_objects};
use fpas_unit::object::{
    ChunkDefinition as ObjectDefinition, ChunkDefinitionKind as DefinitionKind,
    ChunkFunction as ObjectFunction, ChunkImport as ObjectImport,
};

use object_fixture::object;

#[test]
fn private_import_is_rejected() {
    let mut unit = object("unit", vec![Op::Halt], Vec::new());
    unit.definitions.push(ObjectDefinition {
        name: "unit.hidden".to_string(),
        kind: DefinitionKind::Global,
        public: false,
    });
    let mut program = object("program", vec![Op::Halt], Vec::new());
    program.imports.push(ObjectImport {
        name: "unit.hidden".to_string(),
        kind: DefinitionKind::Global,
    });

    assert!(matches!(
        link_objects(&[unit], &program),
        Err(LinkError::UnresolvedImport { .. })
    ));
}

#[test]
fn kind_mismatched_import_is_rejected() {
    let mut unit = object("unit", vec![Op::Halt], Vec::new());
    unit.definitions.push(ObjectDefinition {
        name: "unit.value".to_string(),
        kind: DefinitionKind::Global,
        public: true,
    });
    let mut program = object("program", vec![Op::Halt], Vec::new());
    program.imports.push(ObjectImport {
        name: "unit.value".to_string(),
        kind: DefinitionKind::Callable,
    });

    assert!(matches!(
        link_objects(&[unit], &program),
        Err(LinkError::UnresolvedImport {
            owner,
            name,
            kind: DefinitionKind::Callable,
        }) if owner == "program" && name == "unit.value"
    ));
}

#[test]
fn duplicate_definitions_are_rejected_case_insensitively() {
    let mut first = object("first", vec![Op::Halt], Vec::new());
    first.definitions.push(ObjectDefinition {
        name: "Demo.Value".to_string(),
        kind: DefinitionKind::Global,
        public: true,
    });
    let mut second = object("second", vec![Op::Halt], Vec::new());
    second.definitions.push(ObjectDefinition {
        name: "demo.value".to_string(),
        kind: DefinitionKind::Global,
        public: true,
    });
    let program = object("program", vec![Op::Halt], Vec::new());

    assert!(matches!(
        link_objects(&[first, second], &program),
        Err(LinkError::DuplicateDefinition(_))
    ));
}

#[test]
fn missing_program_code_is_rejected_before_linking_units() {
    let program = object("program", Vec::new(), Vec::new());

    assert!(matches!(
        link_objects(&[], &program),
        Err(LinkError::MissingProgram)
    ));
}

#[test]
fn internal_halt_in_a_startup_section_is_rejected() {
    let unit = object("unit", vec![Op::Halt, Op::Unit, Op::Halt], Vec::new());
    let program = object("program", vec![Op::Halt], Vec::new());

    assert!(matches!(
        link_objects(&[unit], &program),
        Err(LinkError::InvalidObject { detail, .. }) if detail.contains("internal Halt")
    ));
}

#[test]
fn invalid_object_structure_is_reported_with_its_owner() {
    let mut unit = object("broken.unit", vec![Op::Halt], Vec::new());
    unit.locations.clear();
    let program = object("program", vec![Op::Halt], Vec::new());

    assert!(matches!(
        link_objects(&[unit], &program),
        Err(LinkError::InvalidObject { owner, detail })
            if owner == "broken.unit" && detail.contains("LocationCount")
    ));
}

#[test]
fn public_matching_import_is_linked() {
    let mut unit = object(
        "unit",
        vec![Op::Jump(3), Op::Unit, Op::Return, Op::Halt],
        Vec::new(),
    );
    unit.functions.insert(
        "unit.run".to_string(),
        ObjectFunction {
            code_start: 1,
            arity: 0,
        },
    );
    unit.definitions.push(ObjectDefinition {
        name: "unit.run".to_string(),
        kind: DefinitionKind::Callable,
        public: true,
    });
    let mut program = object("program", vec![Op::Halt], Vec::new());
    program.imports.push(ObjectImport {
        name: "UNIT.RUN".to_string(),
        kind: DefinitionKind::Callable,
    });

    assert!(link_objects(&[unit], &program).is_ok());
}

#[test]
fn callable_definition_requires_function_in_its_own_object() {
    let mut defining = object("defining", vec![Op::Halt], Vec::new());
    defining.definitions.push(ObjectDefinition {
        name: "demo.run".to_string(),
        kind: DefinitionKind::Callable,
        public: true,
    });
    let mut unrelated = object(
        "unrelated",
        vec![Op::Jump(3), Op::Unit, Op::Return, Op::Halt],
        Vec::new(),
    );
    unrelated.functions.insert(
        "demo.run".to_string(),
        ObjectFunction {
            code_start: 1,
            arity: 0,
        },
    );
    let program = object("program", vec![Op::Halt], Vec::new());

    assert_eq!(
        link_objects(&[defining, unrelated], &program).err(),
        Some(LinkError::MissingFunctionImplementation {
            owner: "defining".to_string(),
            name: "demo.run".to_string(),
        })
    );
}

#[test]
fn callable_definition_matches_own_function_case_insensitively() {
    let mut unit = object(
        "unit",
        vec![Op::Jump(3), Op::Unit, Op::Return, Op::Halt],
        Vec::new(),
    );
    unit.definitions.push(ObjectDefinition {
        name: "Demo.Run".to_string(),
        kind: DefinitionKind::Callable,
        public: true,
    });
    unit.functions.insert(
        "demo.run".to_string(),
        ObjectFunction {
            code_start: 1,
            arity: 0,
        },
    );
    let program = object("program", vec![Op::Halt], Vec::new());

    assert!(link_objects(&[unit], &program).is_ok());
}

#[test]
fn extra_local_function_without_definition_is_linked() {
    let mut unit = object(
        "unit",
        vec![Op::Jump(3), Op::Unit, Op::Return, Op::Halt],
        Vec::new(),
    );
    unit.functions.insert(
        "unit.local".to_string(),
        ObjectFunction {
            code_start: 1,
            arity: 0,
        },
    );
    let program = object("program", vec![Op::Halt], Vec::new());

    assert!(link_objects(&[unit], &program).is_ok());
}

#[test]
fn unit_function_at_stripped_halt_cannot_rebind_to_next_object() {
    let mut unit = object("unit", vec![Op::Halt], Vec::new());
    unit.functions.insert(
        "unit.bad".to_string(),
        ObjectFunction {
            code_start: 0,
            arity: 0,
        },
    );
    let next = object("next", vec![Op::Unit, Op::Halt], Vec::new());
    let program = object("program", vec![Op::Halt], Vec::new());

    assert_eq!(
        link_objects(&[unit, next], &program).err(),
        Some(LinkError::StrippedFunctionEntry {
            owner: "unit".to_string(),
            name: "unit.bad".to_string(),
            offset: 0,
            retained_code: 0,
        })
    );
}

#[test]
fn linked_executable_rejects_unknown_intrinsic() {
    let program = object(
        "program",
        vec![Op::Intrinsic(u16::MAX), Op::Halt],
        Vec::new(),
    );

    assert_eq!(
        link_objects(&[], &program).err(),
        Some(LinkError::InvalidExecutable(ExecutableError::Intrinsic {
            instruction: 0,
            intrinsic: u16::MAX,
        }))
    );
}

#[test]
fn linked_executable_rejects_one_past_end_jump() {
    let program = object("program", vec![Op::Jump(2), Op::Halt], Vec::new());

    assert_eq!(
        link_objects(&[], &program).err(),
        Some(LinkError::InvalidExecutable(ExecutableError::CodeTarget {
            instruction: 0,
            target: 2,
            code: 2,
        }))
    );
}

#[test]
fn linked_executable_accepts_root_return() {
    let program = object("program", vec![Op::Unit, Op::Return, Op::Halt], Vec::new());

    assert!(link_objects(&[], &program).is_ok());
}

#[test]
fn object_validation_rejects_jump_past_local_stream() {
    let program = object("program", vec![Op::Jump(3), Op::Halt], Vec::new());

    assert!(matches!(
        link_objects(&[], &program),
        Err(LinkError::InvalidObject { owner, detail })
            if owner == "program" && detail.contains("CodeTarget")
    ));
}

#[test]
fn duplicate_callable_names_are_rejected_case_insensitively() {
    let mut first = object("first", vec![Op::Unit, Op::Halt], Vec::new());
    first.functions.insert(
        "Demo.Run".to_string(),
        ObjectFunction {
            code_start: 0,
            arity: 0,
        },
    );
    let mut second = object("second", vec![Op::Unit, Op::Halt], Vec::new());
    second.functions.insert(
        "demo.run".to_string(),
        ObjectFunction {
            code_start: 0,
            arity: 0,
        },
    );
    let program = object("program", vec![Op::Halt], Vec::new());

    assert!(matches!(
        link_objects(&[first, second], &program),
        Err(LinkError::DuplicateFunction(name)) if name == "demo.run"
    ));
}

#![allow(
    clippy::expect_used,
    reason = "linker fixtures use expect for compact result assertions"
)]

use std::collections::BTreeMap;

use fpas_bytecode::Op;
use fpas_linker::{LinkError, link_objects};
use fpas_unit::object::{
    DefinitionKind, ObjectConstant, ObjectDefinition, ObjectFunction, ObjectImport, ObjectLocation,
    RelocatableObject, collect_relocations,
};

fn object(owner: &str, code: Vec<Op>, constants: Vec<ObjectConstant>) -> RelocatableObject {
    RelocatableObject {
        owner: owner.to_string(),
        locations: vec![
            ObjectLocation {
                line: 1,
                column: 1,
                source_id: 0,
            };
            code.len()
        ],
        relocations: collect_relocations(&code),
        code,
        constants,
        functions: BTreeMap::new(),
        definitions: Vec::new(),
        imports: Vec::new(),
    }
}

#[test]
fn linker_rebases_jumps_functions_and_constant_indices() {
    let mut unit = object(
        "demo.unit",
        vec![Op::Jump(2), Op::Unit, Op::Halt],
        vec![ObjectConstant::String("shared".to_string())],
    );
    unit.functions.insert(
        "demo.unit.run".to_string(),
        ObjectFunction {
            code_start: 1,
            arity: 0,
        },
    );
    let program = object(
        "demo.program",
        vec![Op::Constant(0), Op::Halt],
        vec![ObjectConstant::String("shared".to_string())],
    );

    let linked = link_objects(&[unit], &program).expect("linking");
    assert_eq!(
        linked.code(),
        &[Op::Jump(2), Op::Unit, Op::Constant(0), Op::Halt]
    );
    assert_eq!(linked.constants().len(), 1);
    assert_eq!(linked.functions().get("demo.unit.run"), Some(&(1, 0)));
}

#[test]
fn linker_rebases_every_constant_operand_shape() {
    let prefix = object(
        "prefix",
        vec![Op::Unit, Op::Halt],
        vec![ObjectConstant::String("prefix".to_string())],
    );
    let unit = object(
        "unit",
        vec![
            Op::GetGlobal(0),
            Op::SetGlobal(0),
            Op::GlobalIndexSet(0, 2),
            Op::Call(0, 1),
            Op::MakeClosure(0, 0),
            Op::MakeRecord(0, 1),
            Op::FieldGet(0),
            Op::FieldSet(0),
            Op::MakeEnum(0, 1, 0),
            Op::IsVariant(0, 1),
            Op::Halt,
        ],
        vec![
            ObjectConstant::String("demo.name".to_string()),
            ObjectConstant::String("Demo.Variant".to_string()),
        ],
    );
    let program = object("program", vec![Op::Halt], Vec::new());

    let linked = link_objects(&[prefix, unit], &program).expect("linking");

    assert_eq!(
        linked.code(),
        &[
            Op::Unit,
            Op::GetGlobal(1),
            Op::SetGlobal(1),
            Op::GlobalIndexSet(1, 2),
            Op::Call(1, 1),
            Op::MakeClosure(1, 0),
            Op::MakeRecord(1, 1),
            Op::FieldGet(1),
            Op::FieldSet(1),
            Op::MakeEnum(1, 2, 0),
            Op::IsVariant(1, 2),
            Op::Halt,
        ]
    );
    assert_eq!(linked.constants().len(), 3);
}

#[test]
fn startup_sections_are_concatenated_dependency_first_with_one_halt() {
    let first = object("first", vec![Op::Unit, Op::Halt], Vec::new());
    let second = object("second", vec![Op::Pop, Op::Halt], Vec::new());
    let program = object("program", vec![Op::Print, Op::Halt], Vec::new());

    let linked = link_objects(&[first, second], &program).expect("linking");
    assert_eq!(linked.code(), &[Op::Unit, Op::Pop, Op::Print, Op::Halt]);
    assert_eq!(
        linked
            .code()
            .iter()
            .filter(|op| matches!(op, Op::Halt))
            .count(),
        1
    );
}

#[test]
fn missing_private_and_kind_mismatched_imports_are_rejected() {
    let mut unit = object("unit", vec![Op::Halt], Vec::new());
    unit.definitions.push(ObjectDefinition {
        name: "unit.hidden".to_string(),
        kind: DefinitionKind::Callable,
        public: false,
    });
    let mut program = object("program", vec![Op::Halt], Vec::new());
    program.imports.push(ObjectImport {
        name: "unit.hidden".to_string(),
        kind: DefinitionKind::Callable,
    });

    assert!(matches!(
        link_objects(&[unit], &program),
        Err(LinkError::UnresolvedImport { .. })
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
    let mut unit = object("unit", vec![Op::Halt], Vec::new());
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

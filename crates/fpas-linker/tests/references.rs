//! Var ABI and selected record layout relocation survive unit linking.

mod common;

use fpas_bytecode::{Instruction, NO_REGISTER, Opcode};
use fpas_linker::{LinkError, link_objects};
use fpas_unit::object::{
    DefinitionTarget, ImportShape, ObjectDefinition, Relocation, RelocationKind, SymbolReference,
};

#[test]
fn imported_callable_modes_are_preserved_and_mismatches_rejected() {
    let mut library = common::unit(true);
    library.functions[0].arity = 1;
    library.functions[0].register_count = 1;
    library.functions[0].var_parameters = vec![0];
    library
        .debug_types
        .push(fpas_unit::object::ObjectDebugType::Reference(0));
    let mut program = common::program();
    program.functions[0].register_count = 1;
    program.functions[0].code[0] = Instruction::abc(Opcode::CallDirect, NO_REGISTER, 0, 0, 1)
        .unwrap()
        .word();
    program.imports[0].shape = ImportShape::Function {
        arity: 1,
        var_parameters: vec![0],
        capture_count: 0,
        returns_value: false,
    };
    let linked = link_objects(&[library.clone()], &program).unwrap();
    assert!(
        linked
            .executable()
            .functions
            .iter()
            .any(|function| function.arity == 1 && function.var_parameters == vec![0])
    );
    assert!(
        linked
            .executable()
            .debug_types
            .iter()
            .any(|ty| matches!(ty, fpas_bytecode::DebugType::Reference(_)))
    );
    program.imports[0].shape = ImportShape::Function {
        arity: 1,
        var_parameters: Vec::new(),
        capture_count: 0,
        returns_value: false,
    };
    assert!(matches!(
        link_objects(&[library], &program),
        Err(LinkError::IncompatibleImport { .. })
    ));
}

#[test]
fn selected_field_relocates_layout_while_preserving_its_field_slot() {
    let mut library = common::unit(true);
    for name in ["library.unit.zrecord", "library.unit.arecord"] {
        let index = library.records.len() as u32;
        let mut layout = common::one_field_record(0);
        layout.name = name.to_string();
        library.records.push(layout);
        library.definitions.push(ObjectDefinition {
            name: name.to_string(),
            target: DefinitionTarget::Record(index),
            public: true,
        });
    }
    library
        .definitions
        .sort_by(|left, right| left.name.cmp(&right.name));
    library.functions[0].register_count = 1;
    library.functions[0].code.insert(
        0,
        Instruction::abc(Opcode::SelectReferenceField, 0, 0, 0, 0)
            .unwrap()
            .word(),
    );
    library.relocations.push(Relocation {
        function: 0,
        instruction: 0,
        kind: RelocationKind::Record(SymbolReference::Local(0)),
    });
    let linked = link_objects(&[library], &common::program()).unwrap();
    let image = linked.executable();
    let word = image
        .code
        .iter()
        .find(|word| word.opcode() == Ok(Opcode::SelectReferenceField))
        .unwrap()
        .abc_payload();
    assert_eq!(word.b, 1);
    assert_eq!(word.c, 0);
    assert_eq!(
        image.strings.get(image.records[usize::from(word.b)].name),
        Some("library.unit.zrecord")
    );
}

#[test]
fn object_function_and_import_parameter_modes_reject_duplicates_and_out_of_range_positions() {
    for positions in [vec![0, 0], vec![1], vec![u8::MAX]] {
        let mut program = common::program();
        program.imports[0].shape = ImportShape::Function {
            arity: 1,
            var_parameters: positions.clone(),
            capture_count: 0,
            returns_value: false,
        };
        assert!(matches!(
            link_objects(&[common::unit(true)], &program),
            Err(LinkError::InvalidObject { .. })
        ));
        let mut library = common::unit(true);
        library.functions[0].arity = 1;
        library.functions[0].register_count = 1;
        library.functions[0].var_parameters = positions;
        assert!(matches!(
            link_objects(&[library], &common::program()),
            Err(LinkError::InvalidObject { .. })
        ));
    }
}

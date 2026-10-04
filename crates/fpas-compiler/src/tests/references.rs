//! Typed reference selection, source mapping and unit-object integration.

use fpas_ir::*;

mod arrays;
mod source_calls;

fn instruction(id: u32, ty: u32, operation: Operation) -> Instruction {
    Instruction {
        source: None,
        result: Some(ValueDefinition {
            id: ValueId::new(id),
            ty: TypeId::new(ty),
        }),
        operation,
    }
}

fn routine(
    id: u32,
    parameters: Vec<TypeId>,
    result: TypeId,
    instructions: Vec<Instruction>,
    returned: Option<ValueId>,
) -> Function {
    Function {
        id: FunctionId::new(id),
        name: format!("references.routine{id}"),
        signature: FunctionSignature {
            parameters: parameters.clone(),
            result,
        },
        parameters: parameters
            .into_iter()
            .enumerate()
            .map(|(index, ty)| ValueDefinition {
                id: ValueId::new(index as u32),
                ty,
            })
            .collect(),
        locals: Vec::new(),
        captures: Vec::new(),
        debug: FunctionDebugInfo::default(),
        blocks: vec![BasicBlock {
            id: BlockId::new(0),
            parameters: Vec::new(),
            instructions,
            terminators: vec![Terminator::Return(returned)],
        }],
        entry: BlockId::new(0),
        max_call_arguments: 1,
        can_spawn_tasks: false,
    }
}

fn program() -> Program {
    let reference = |operation| Operation::Reference(operation);
    let mut root = routine(
        0,
        Vec::new(),
        TypeId::new(0),
        vec![
            instruction(1, 1, Operation::Const(Constant::Integer(1))),
            instruction(
                2,
                2,
                Operation::MakeRecord {
                    layout: RecordLayoutId::new(0),
                    fields: vec![ValueId::new(1)],
                },
            ),
            instruction(3, 3, Operation::MakeCell(ValueId::new(2))),
            instruction(
                4,
                4,
                reference(ReferenceOperation::Reserve(ValueId::new(3))),
            ),
            instruction(
                5,
                5,
                reference(ReferenceOperation::Field {
                    reference: ValueId::new(4),
                    layout: RecordLayoutId::new(0),
                    field: FieldId::new(0),
                }),
            ),
            instruction(6, 2, reference(ReferenceOperation::Read(ValueId::new(4)))),
            instruction(
                7,
                0,
                Operation::CallDirect {
                    function: FunctionId::new(1),
                    arguments: vec![ValueId::new(5)],
                },
            ),
            instruction(8, 2, Operation::CellRead(ValueId::new(3))),
            instruction(
                9,
                1,
                Operation::LoadField {
                    record: ValueId::new(8),
                    layout: RecordLayoutId::new(0),
                    field: FieldId::new(0),
                },
            ),
        ],
        None,
    );
    // Retain the whole-root snapshot past field selection, forcing distinct SSA locations.
    root.blocks[0].instructions.insert(
        8,
        instruction(
            10,
            1,
            Operation::LoadField {
                record: ValueId::new(6),
                layout: RecordLayoutId::new(0),
                field: FieldId::new(0),
            },
        ),
    );
    root.blocks[0].instructions.extend([
        instruction(11, 1, Operation::Const(Constant::Integer(9))),
        instruction(
            12,
            7,
            Operation::Binary {
                operation: BinaryOperation::Equal,
                left: ValueId::new(9),
                right: ValueId::new(11),
            },
        ),
        instruction(
            13,
            7,
            Operation::Binary {
                operation: BinaryOperation::Equal,
                left: ValueId::new(10),
                right: ValueId::new(1),
            },
        ),
        instruction(
            14,
            7,
            Operation::Binary {
                operation: BinaryOperation::AndBoolean,
                left: ValueId::new(12),
                right: ValueId::new(13),
            },
        ),
    ]);
    root.blocks[0].terminators = vec![Terminator::Branch {
        condition: ValueId::new(14),
        then_target: BlockTarget {
            block: BlockId::new(1),
            arguments: Vec::new(),
        },
        else_target: BlockTarget {
            block: BlockId::new(2),
            arguments: Vec::new(),
        },
    }];
    root.blocks.extend([
        BasicBlock {
            id: BlockId::new(1),
            parameters: Vec::new(),
            instructions: Vec::new(),
            terminators: vec![Terminator::Return(None)],
        },
        BasicBlock {
            id: BlockId::new(2),
            parameters: Vec::new(),
            instructions: vec![instruction(
                20,
                6,
                Operation::Const(Constant::String(
                    "reference write or snapshot failed".to_string(),
                )),
            )],
            terminators: vec![Terminator::Panic(ValueId::new(20))],
        },
    ]);
    let mut callee = routine(
        1,
        vec![TypeId::new(5)],
        TypeId::new(0),
        vec![instruction(1, 1, Operation::Const(Constant::Integer(9)))],
        None,
    );
    callee.blocks[0].instructions.push(Instruction {
        source: None,
        result: None,
        operation: reference(ReferenceOperation::Write {
            reference: ValueId::new(0),
            value: ValueId::new(1),
        }),
    });
    Program {
        types: vec![
            IrType::Unit,
            IrType::Integer,
            IrType::Record(RecordLayoutId::new(0)),
            IrType::Cell(TypeId::new(2)),
            IrType::Reference(TypeId::new(2)),
            IrType::Reference(TypeId::new(1)),
            IrType::String,
            IrType::Boolean,
        ]
        .into_iter()
        .enumerate()
        .map(|(index, kind)| TypeDefinition {
            id: TypeId::new(index as u32),
            kind,
        })
        .collect(),
        globals: Vec::new(),
        record_layouts: vec![RecordLayout {
            id: RecordLayoutId::new(0),
            name: "references.point".to_string(),
            fields: vec![RecordField {
                id: FieldId::new(0),
                name: "value".to_string(),
                ty: TypeId::new(1),
            }],
        }],
        enum_layouts: Vec::new(),
        intrinsics: Vec::new(),
        functions: vec![root, callee],
        entry: FunctionId::new(0),
    }
}

#[test]
fn reference_selection_preserves_root_path_identity_and_callee_modes() {
    let executable = crate::bytecode::compile_program(program()).unwrap();
    assert_eq!(executable.executable().functions[1].var_parameters, vec![0]);
    assert!(
        executable
            .executable()
            .code
            .iter()
            .any(|word| word.opcode() == Ok(fpas_bytecode::Opcode::SelectReferenceField))
    );
    assert_eq!(
        fpas_vm::Vm::new(executable).run().unwrap().value,
        fpas_bytecode::Value::Unit
    );
}

#[test]
fn reference_selection_is_deterministic_and_survives_object_codec_and_linking() {
    let first = crate::bytecode::compile_program(program()).unwrap();
    assert_eq!(first, crate::bytecode::compile_program(program()).unwrap());
    let object =
        fpas_unit::object::RelocatableObject::from_executable("references", first).unwrap();
    let decoded =
        fpas_unit::object::decode_object(&fpas_unit::object::encode_object(&object).unwrap())
            .unwrap();
    let linked = fpas_linker::link_objects(&[], &decoded).unwrap();
    assert_eq!(
        fpas_vm::Vm::new(linked).run().unwrap().value,
        fpas_bytecode::Value::Unit
    );
}

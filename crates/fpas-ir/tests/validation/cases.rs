#[test]
fn empty_unit_root_function_is_valid() {
    assert!(scalar_program().validate().is_ok());
}

#[test]
fn typed_scalar_aggregate_call_closure_intrinsic_and_task_operations_are_valid() {
    assert!(all_operations_program().validate().is_ok());
}

#[test]
fn branch_with_block_parameters_is_valid() {
    let mut program = scalar_program();
    program.functions[0] = root(vec![
        BasicBlock {
            id: BlockId::new(0),
            parameters: Vec::new(),
            instructions: vec![
                Instruction {
                    source: None,
                    result: Some(value(1, BOOLEAN)),
                    operation: Operation::Const(Constant::Boolean(true)),
                },
                Instruction {
                    source: None,
                    result: Some(value(2, INTEGER)),
                    operation: Operation::Const(Constant::Integer(1)),
                },
            ],
            terminators: vec![Terminator::Branch {
                condition: ValueId::new(1),
                then_target: BlockTarget {
                    block: BlockId::new(1),
                    arguments: vec![ValueId::new(2)],
                },
                else_target: BlockTarget {
                    block: BlockId::new(2),
                    arguments: vec![ValueId::new(2)],
                },
            }],
        },
        BasicBlock {
            id: BlockId::new(1),
            parameters: vec![BlockParameter {
                id: ValueId::new(3),
                ty: INTEGER,
            }],
            instructions: Vec::new(),
            terminators: vec![Terminator::Return(None)],
        },
        BasicBlock {
            id: BlockId::new(2),
            parameters: vec![BlockParameter {
                id: ValueId::new(4),
                ty: INTEGER,
            }],
            instructions: Vec::new(),
            terminators: vec![Terminator::Return(None)],
        },
    ]);
    assert!(program.validate().is_ok());
}

#[test]
fn loop_backedge_with_block_parameter_is_valid() {
    let mut program = scalar_program();
    program.functions[0] = root(vec![
        BasicBlock {
            id: BlockId::new(0),
            parameters: Vec::new(),
            instructions: vec![Instruction {
                source: None,
                result: Some(value(1, INTEGER)),
                operation: Operation::Const(Constant::Integer(0)),
            }],
            terminators: vec![Terminator::Jump(BlockTarget {
                block: BlockId::new(1),
                arguments: vec![ValueId::new(1)],
            })],
        },
        BasicBlock {
            id: BlockId::new(1),
            parameters: vec![BlockParameter {
                id: ValueId::new(2),
                ty: INTEGER,
            }],
            instructions: vec![Instruction {
                source: None,
                result: Some(value(3, BOOLEAN)),
                operation: Operation::Const(Constant::Boolean(false)),
            }],
            terminators: vec![Terminator::Branch {
                condition: ValueId::new(3),
                then_target: BlockTarget {
                    block: BlockId::new(1),
                    arguments: vec![ValueId::new(2)],
                },
                else_target: BlockTarget {
                    block: BlockId::new(2),
                    arguments: Vec::new(),
                },
            }],
        },
        BasicBlock {
            id: BlockId::new(2),
            parameters: Vec::new(),
            instructions: Vec::new(),
            terminators: vec![Terminator::Return(None)],
        },
    ]);
    assert!(program.validate().is_ok());
}

#[test]
fn semantic_instruction_preserves_source_span() {
    let span = SourceSpan::new_with_source(7, 3, 2, 5, 11);
    let mut program = scalar_program();
    program.functions[0].blocks[0].instructions = vec![Instruction {
        source: Some(span),
        result: Some(value(1, INTEGER)),
        operation: Operation::Const(Constant::Integer(1)),
    }];

    assert!(program.validate().is_ok());
    assert_eq!(
        program.functions[0].blocks[0].instructions[0].source,
        Some(span)
    );
}

#[test]
fn duplicate_type_identifier_is_rejected() {
    let mut program = scalar_program();
    program.types.push(TypeDefinition {
        id: UNIT,
        kind: IrType::Unit,
    });
    assert!(
        matches!(program.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::DuplicateId { .. }))
    );
}

#[test]
fn duplicate_function_block_value_and_local_identifiers_are_rejected() {
    let mut function = scalar_program();
    function.functions.push(root(vec![return_unit_block()]));
    assert!(function.validate().is_err());

    let mut block = scalar_program();
    block.functions[0].blocks.push(return_unit_block());
    assert!(block.validate().is_err());

    let mut defined_value = scalar_program();
    defined_value.functions[0].blocks[0].instructions = vec![
        Instruction {
            source: None,
            result: Some(value(1, INTEGER)),
            operation: Operation::Const(Constant::Integer(1)),
        },
        Instruction {
            source: None,
            result: Some(value(1, INTEGER)),
            operation: Operation::Const(Constant::Integer(2)),
        },
    ];
    assert!(defined_value.validate().is_err());

    let mut local = scalar_program();
    local.functions[0].locals.push(Local {
        id: LocalId::new(0),
        ty: INTEGER,
        mutable: false,
        capture: None,
    });
    assert!(local.validate().is_err());
}

#[test]
fn unknown_function_block_value_local_and_type_are_rejected() {
    let mut program = scalar_program();
    program.entry = FunctionId::new(99);
    assert!(program.validate().is_err());
    program.entry = FunctionId::new(0);
    program.functions[0].entry = BlockId::new(99);
    assert!(program.validate().is_err());
    program.functions[0].entry = BlockId::new(0);
    program.functions[0].blocks[0].instructions = vec![Instruction {
        source: None,
        result: Some(value(1, INTEGER)),
        operation: Operation::ReadLocal(LocalId::new(99)),
    }];
    assert!(program.validate().is_err());
    program.functions[0].blocks[0].instructions = vec![Instruction {
        source: None,
        result: Some(value(1, TypeId::new(99))),
        operation: Operation::Const(Constant::Integer(1)),
    }];
    assert!(program.validate().is_err());
    program.functions[0].blocks[0].instructions = vec![Instruction {
        source: None,
        result: Some(value(1, INTEGER)),
        operation: Operation::Binary {
            operation: BinaryOperation::AddInteger,
            left: ValueId::new(88),
            right: ValueId::new(88),
        },
    }];
    assert!(program.validate().is_err());
}

#[test]
fn value_use_before_definition_is_rejected() {
    let mut program = scalar_program();
    program.functions[0].blocks[0].instructions = vec![
        Instruction {
            source: None,
            result: Some(value(2, INTEGER)),
            operation: Operation::Binary {
                operation: BinaryOperation::AddInteger,
                left: ValueId::new(1),
                right: ValueId::new(1),
            },
        },
        Instruction {
            source: None,
            result: Some(value(1, INTEGER)),
            operation: Operation::Const(Constant::Integer(1)),
        },
    ];
    assert!(
        matches!(program.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::UseBeforeDefinition { .. }))
    );
}

#[test]
fn missing_and_multiple_terminators_are_rejected() {
    let mut missing = scalar_program();
    missing.functions[0].blocks[0].terminators.clear();
    assert!(
        matches!(missing.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::MissingTerminator))
    );
    let mut multiple = scalar_program();
    multiple.functions[0].blocks[0]
        .terminators
        .push(Terminator::Return(None));
    assert!(
        matches!(multiple.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::MultipleTerminators { .. }))
    );
}

#[test]
fn invalid_block_target_and_unreachable_block_are_rejected() {
    let mut missing_target = scalar_program();
    missing_target.functions[0].blocks[0].terminators = vec![Terminator::Jump(BlockTarget {
        block: BlockId::new(99),
        arguments: Vec::new(),
    })];
    assert!(missing_target.validate().is_err());
    let mut unreachable = scalar_program();
    unreachable.functions[0].blocks.push(BasicBlock {
        id: BlockId::new(1),
        parameters: Vec::new(),
        instructions: Vec::new(),
        terminators: vec![Terminator::Return(None)],
    });
    assert!(
        matches!(unreachable.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::UnreachableBlock { .. }))
    );
}

#[test]
fn block_argument_count_and_type_mismatches_are_rejected() {
    let mut wrong_count = scalar_program();
    wrong_count.functions[0] = root(vec![
        BasicBlock {
            id: BlockId::new(0),
            parameters: Vec::new(),
            instructions: Vec::new(),
            terminators: vec![Terminator::Jump(BlockTarget {
                block: BlockId::new(1),
                arguments: Vec::new(),
            })],
        },
        BasicBlock {
            id: BlockId::new(1),
            parameters: vec![BlockParameter {
                id: ValueId::new(1),
                ty: INTEGER,
            }],
            instructions: Vec::new(),
            terminators: vec![Terminator::Return(None)],
        },
    ]);
    assert!(
        matches!(wrong_count.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::BlockArgumentCount { .. }))
    );
    let mut wrong_type = wrong_count;
    wrong_type.functions[0].blocks[0].instructions = vec![Instruction {
        source: None,
        result: Some(value(2, BOOLEAN)),
        operation: Operation::Const(Constant::Boolean(true)),
    }];
    wrong_type.functions[0].blocks[0].terminators = vec![Terminator::Jump(BlockTarget {
        block: BlockId::new(1),
        arguments: vec![ValueId::new(2)],
    })];
    assert!(
        matches!(wrong_type.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::BlockArgumentType { .. }))
    );
}

#[test]
fn operand_result_direct_call_return_capture_and_layout_errors_are_rejected() {
    let mut operand = all_operations_program();
    operand.functions[0].blocks[0].instructions[5].result = Some(value(5, BOOLEAN));
    assert!(operand.validate().is_err());
    let mut direct_call = all_operations_program();
    direct_call.functions[0].blocks[0].instructions[6].operation = Operation::CallDirect {
        function: FunctionId::new(1),
        arguments: Vec::new(),
    };
    assert!(
        matches!(direct_call.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::DirectCallSignature { .. }))
    );
    let mut returned = all_operations_program();
    returned.functions[1].blocks[0].terminators = vec![Terminator::Return(None)];
    assert!(returned.validate().is_err());
    let mut capture = all_operations_program();
    capture.functions[1].captures = vec![CaptureDeclaration {
        ty: INTEGER,
        kind: CaptureKind::Value,
    }];
    assert!(
        matches!(capture.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::ClosureCaptureCount { expected: 1, actual: 0 }))
    );
    let mut cell_capture = all_operations_program();
    cell_capture.functions[1].captures = vec![CaptureDeclaration {
        ty: INTEGER,
        kind: CaptureKind::Cell,
    }];
    cell_capture.functions[0].blocks[0].instructions[7].operation = Operation::MakeClosure {
        function: FunctionId::new(1),
        captures: vec![ValueId::new(1)],
    };
    assert!(
        matches!(cell_capture.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::ClosureCaptureType { index: 0, .. }))
    );
    let mut layout = all_operations_program();
    layout.functions[0].blocks[0].instructions[11].operation = Operation::LoadField {
        record: ValueId::new(10),
        layout: RecordLayoutId::new(99),
        field: FieldId::new(0),
    };
    assert!(layout.validate().is_err());
}

#[test]
fn result_shape_and_cell_type_errors_are_rejected() {
    let mut missing = scalar_program();
    missing.functions[0].blocks[0].instructions = vec![Instruction {
        source: None,
        result: None,
        operation: Operation::Const(Constant::Integer(1)),
    }];
    assert!(
        matches!(missing.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::MissingResult))
    );
    let mut unexpected = scalar_program();
    unexpected.functions[0].blocks[0].instructions = vec![Instruction {
        source: None,
        result: Some(value(1, UNIT)),
        operation: Operation::Yield,
    }];
    assert!(
        matches!(unexpected.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::UnexpectedResult))
    );
}

#[test]
fn maximum_ids_and_checked_count_boundaries_are_portable() {
    assert_eq!(FunctionId::MAX.get(), u32::MAX);
    assert_eq!(BlockId::MAX.get(), u32::MAX);
    assert_eq!(ValueId::MAX.get(), u32::MAX);
    assert_eq!(LocalId::MAX.get(), u32::MAX);
    assert_eq!(TypeId::MAX.get(), u32::MAX);
    assert_eq!(GlobalId::MAX.get(), u32::MAX);
    assert_eq!(RecordLayoutId::MAX.get(), u32::MAX);
    assert_eq!(EnumLayoutId::MAX.get(), u32::MAX);
    assert_eq!(FieldId::MAX.get(), u32::MAX);
    assert_eq!(VariantId::MAX.get(), u32::MAX);
    assert_eq!(IntrinsicId::MAX.get(), u32::MAX);
    assert!(FunctionId::try_from_index(u32::MAX as usize).is_ok());
    assert!(checked_count("test count", u32::MAX as usize).is_ok());
    if usize::BITS > 32 {
        assert!(FunctionId::try_from_index(u32::MAX as usize + 1).is_err());
        assert!(checked_count("test count", u32::MAX as usize + 1).is_err());
    }
}

#[test]
fn maximum_block_identifier_is_valid() {
    let mut program = scalar_program();
    program.functions[0].entry = BlockId::MAX;
    program.functions[0].blocks[0].id = BlockId::MAX;

    assert!(program.validate().is_ok());
}

#[test]
fn validation_preserves_declared_function_and_block_order() {
    let mut program = all_operations_program();
    program.functions.swap(0, 1);
    program.entry = FunctionId::new(0);
    let function_ids = program
        .functions
        .iter()
        .map(|function| function.id.get())
        .collect::<Vec<_>>();
    assert!(program.validate().is_ok());
    assert_eq!(
        function_ids,
        program
            .functions
            .iter()
            .map(|function| function.id.get())
            .collect::<Vec<_>>()
    );
}

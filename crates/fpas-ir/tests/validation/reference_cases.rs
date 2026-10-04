fn reference_program() -> Program {
    let mut program = scalar_program();
    program.types.push(TypeDefinition {
        id: TypeId::new(15),
        kind: IrType::Reference(INTEGER),
    });
    program.functions[0].blocks[0].instructions = vec![
        Instruction {
            source: None,
            result: Some(value(1, INTEGER)),
            operation: Operation::Const(Constant::Integer(1)),
        },
        Instruction {
            source: None,
            result: Some(value(2, CELL)),
            operation: Operation::MakeCell(ValueId::new(1)),
        },
        Instruction {
            source: None,
            result: Some(value(3, TypeId::new(15))),
            operation: Operation::Reference(fpas_ir::ReferenceOperation::Reserve(ValueId::new(2))),
        },
        Instruction {
            source: None,
            result: Some(value(4, INTEGER)),
            operation: Operation::Reference(fpas_ir::ReferenceOperation::Read(ValueId::new(3))),
        },
        Instruction {
            source: None,
            result: None,
            operation: Operation::Reference(fpas_ir::ReferenceOperation::Write {
                reference: ValueId::new(3),
                value: ValueId::new(4),
            }),
        },
        Instruction {
            source: None,
            result: None,
            operation: Operation::Reference(fpas_ir::ReferenceOperation::Release(ValueId::new(3))),
        },
    ];
    program
}

#[test]
fn selected_reference_ir_validates_reads_writes_release_and_operand_types() {
    assert!(reference_program().validate().is_ok());
    for (instruction, ty) in [(2, INTEGER), (3, BOOLEAN)] {
        let mut program = reference_program();
        program.functions[0].blocks[0].instructions[instruction]
            .result
            .as_mut()
            .unwrap()
            .ty = ty;
        assert!(program.validate().is_err());
    }
    let mut program = reference_program();
    program.functions[0].blocks[0].instructions[4].operation =
        Operation::Reference(fpas_ir::ReferenceOperation::Write {
            reference: ValueId::new(2),
            value: ValueId::new(1),
        });
    assert!(program.validate().is_err());
}

#[test]
fn reference_storage_and_results_are_rejected_while_callable_var_modes_are_values() {
    for kind in [
        IrType::Array(TypeId::new(15)),
        IrType::Option(TypeId::new(15)),
        IrType::Cell(TypeId::new(15)),
        IrType::Task(TypeId::new(15)),
    ] {
        let mut program = reference_program();
        program.types.push(TypeDefinition {
            id: TypeId::new(16),
            kind,
        });
        assert!(
            matches!(program.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::ReferenceEscape { .. }))
        );
    }
    let mut program = reference_program();
    program.types.push(TypeDefinition {
        id: TypeId::new(16),
        kind: IrType::Function {
            parameters: vec![TypeId::new(15)],
            result: INTEGER,
        },
    });
    assert!(program.validate().is_ok());
    program.functions[0].signature.result = TypeId::new(15);
    assert!(
        matches!(program.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::ReferenceEscape { context: "callable result", .. }))
    );
}

#[test]
fn reference_parameter_is_not_a_dynamic_value_argument() {
    let mut program = reference_program();
    let mut callee = root(vec![return_unit_block()]);
    callee.id = FunctionId::new(1);
    callee.signature.parameters = vec![DYNAMIC];
    callee.parameters = vec![value(0, DYNAMIC)];
    program.functions.push(callee);
    program.functions[0].blocks[0]
        .instructions
        .push(Instruction {
            source: None,
            result: Some(value(5, UNIT)),
            operation: Operation::CallDirect {
                function: FunctionId::new(1),
                arguments: vec![ValueId::new(3)],
            },
        });
    assert!(program.validate().is_err());
}

#[test]
fn reference_spill_locals_validate_but_captured_storage_remains_forbidden() {
    let mut program = reference_program();
    program.functions[0].locals.push(Local {
        id: LocalId::new(2),
        ty: TypeId::new(15),
        mutable: true,
        capture: None,
    });
    program.functions[0].blocks[0].instructions.insert(
        3,
        Instruction {
            source: None,
            result: None,
            operation: Operation::WriteLocal {
                local: LocalId::new(2),
                value: ValueId::new(3),
            },
        },
    );
    program.functions[0].blocks[0].instructions.insert(
        4,
        Instruction {
            source: None,
            result: Some(value(5, TypeId::new(15))),
            operation: Operation::ReadLocal(LocalId::new(2)),
        },
    );
    assert!(program.validate().is_ok());
    program.functions[0].locals[2].capture = Some(CaptureKind::Value);
    assert!(
        matches!(program.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::ReferenceEscape { context: "local storage", .. }))
    );
}

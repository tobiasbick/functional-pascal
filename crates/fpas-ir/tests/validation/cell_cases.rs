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

    let mut valid_cell = scalar_program();
    valid_cell.functions[0].blocks[0].instructions = vec![
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
    ];
    assert!(valid_cell.validate().is_ok());

    let mut invalid_cell = valid_cell;
    invalid_cell.functions[0].blocks[0].instructions[1].result = Some(value(2, BOOLEAN));
    assert!(
        matches!(invalid_cell.validate(), Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::OperandType { operand: "cell result", .. }))
    );
}

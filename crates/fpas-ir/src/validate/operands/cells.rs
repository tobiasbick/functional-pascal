fn validate_cell_make(
    scope: OperandScope<'_>,
    value: ValueId,
    result: Option<ValueDefinition>,
) -> Result<(), ValidationError> {
    let OperandScope {
        program,
        function,
        block,
        instruction,
        all_values,
        available,
    } = scope;
    let value_ty = value_type(function, block, instruction, value, all_values, available)?;
    let result = result.ok_or_else(|| {
        function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::MissingResult,
        )
    })?;
    match program.ty(result.ty).map(|definition| &definition.kind) {
        Some(IrType::Cell(inner)) if *inner == value_ty => Ok(()),
        _ => Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::OperandType {
                operand: "cell result",
                expected: value_ty.get(),
                actual: result.ty.get(),
            },
        )),
    }
}

fn validate_cell_read(
    scope: OperandScope<'_>,
    cell: ValueId,
    result: Option<ValueDefinition>,
) -> Result<(), ValidationError> {
    let OperandScope {
        program,
        function,
        block,
        instruction,
        all_values,
        available,
    } = scope;
    let cell_ty = value_type(function, block, instruction, cell, all_values, available)?;
    let Some(IrType::Cell(inner)) = program.ty(cell_ty).map(|definition| &definition.kind) else {
        return Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::TypeCategory {
                operand: "cell",
                expected: "cell",
                actual: cell_ty.get(),
            },
        ));
    };
    require_result_type(function, block, instruction, result, *inner)
}

fn validate_cell_write(
    scope: OperandScope<'_>,
    cell: ValueId,
    value: ValueId,
) -> Result<(), ValidationError> {
    let OperandScope {
        program,
        function,
        block,
        instruction,
        all_values,
        available,
    } = scope;
    let cell_ty = value_type(function, block, instruction, cell, all_values, available)?;
    let value_ty = value_type(function, block, instruction, value, all_values, available)?;
    let Some(IrType::Cell(inner)) = program.ty(cell_ty).map(|definition| &definition.kind) else {
        return Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::TypeCategory {
                operand: "cell",
                expected: "cell",
                actual: cell_ty.get(),
            },
        ));
    };
    require_exact(function, block, instruction, "cell value", *inner, value_ty)
}

fn validate_spawn(
    scope: OperandScope<'_>,
    callee: ValueId,
    arguments: &[ValueId],
    result: Option<ValueDefinition>,
) -> Result<(), ValidationError> {
    let OperandScope {
        program,
        function,
        block,
        instruction,
        all_values,
        available,
    } = scope;
    let callee_ty = value_type(function, block, instruction, callee, all_values, available)?;
    let (parameters, output) = function_value_signature(
        function,
        block,
        instruction,
        callee_ty,
        program.ty(callee_ty).map(|definition| &definition.kind),
    )?;
    validate_arguments(scope, arguments, parameters)?;
    let result = result.ok_or_else(|| {
        function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::MissingResult,
        )
    })?;
    match program.ty(result.ty).map(|definition| &definition.kind) {
        Some(IrType::Task(inner))
            if *inner == output
                || matches!(program.ty(*inner).map(|definition| &definition.kind), Some(IrType::Dynamic)) => Ok(()),
        _ => Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::OperandType {
                operand: "task result",
                expected: output.get(),
                actual: result.ty.get(),
            },
        )),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_cell_make(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    value: ValueId,
    result: Option<ValueDefinition>,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
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

#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_cell_read(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    cell: ValueId,
    result: Option<ValueDefinition>,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
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

#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_cell_write(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    cell: ValueId,
    value: ValueId,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
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

#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_spawn(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    callee: ValueId,
    arguments: &[ValueId],
    result: Option<ValueDefinition>,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
    let callee_ty = value_type(function, block, instruction, callee, all_values, available)?;
    let Some(IrType::Function {
        parameters,
        result: output,
    }) = program.ty(callee_ty).map(|definition| &definition.kind)
    else {
        return Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::CallValueType {
                actual: callee_ty.get(),
            },
        ));
    };
    validate_arguments(
        program,
        function,
        block,
        instruction,
        arguments,
        parameters,
        all_values,
        available,
    )?;
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
            if *inner == *output
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

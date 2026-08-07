#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_direct_call(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    target: FunctionId,
    arguments: &[ValueId],
    result: Option<ValueDefinition>,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
    let target = program.function(target).ok_or_else(|| {
        unknown(
            function,
            block,
            instruction,
            EntityKind::Function,
            target.get(),
        )
    })?;
    validate_arguments(
        program,
        function,
        block,
        instruction,
        arguments,
        &target.signature.parameters,
        all_values,
        available,
    )?;
    require_result_type(
        function,
        block,
        instruction,
        result,
        target.signature.result,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_call_value(
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
    let definition = program.ty(callee_ty).ok_or_else(|| {
        unknown(
            function,
            block,
            instruction,
            EntityKind::Type,
            callee_ty.get(),
        )
    })?;
    let IrType::Function {
        parameters,
        result: expected_result,
    } = &definition.kind
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
    if result.is_some() {
        require_result_type(function, block, instruction, result, *expected_result)?;
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_arguments(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    arguments: &[ValueId],
    parameters: &[TypeId],
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
    if arguments.len() != parameters.len() {
        return Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::DirectCallSignature {
                expected: parameters.len(),
                actual: arguments.len(),
            },
        ));
    }
    for (argument, expected) in arguments.iter().zip(parameters) {
        let actual = value_type(
            function,
            block,
            instruction,
            *argument,
            all_values,
            available,
        )?;
        require_exact(
            function,
            block,
            instruction,
            "call argument",
            *expected,
            actual,
        )?;
    }
    let _ = program;
    Ok(())
}

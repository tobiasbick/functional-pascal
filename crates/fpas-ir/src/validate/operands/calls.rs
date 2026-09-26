fn validate_direct_call(
    scope: OperandScope<'_>,
    target: FunctionId,
    arguments: &[ValueId],
    result: Option<ValueDefinition>,
) -> Result<(), ValidationError> {
    let OperandScope {
        program,
        function,
        block,
        instruction,
        ..
    } = scope;
    let target = program.function(target).ok_or_else(|| {
        unknown(
            function,
            block,
            instruction,
            EntityKind::Function,
            target.get(),
        )
    })?;
    validate_arguments(scope, arguments, &target.signature.parameters)?;
    let result = result.ok_or_else(|| function_error(function.id, Some(block), Some(instruction), ValidationErrorKind::MissingResult))?;
    if types_compatible(program, target.signature.result, result.ty) {
        return Ok(());
    }
    require_result_type(function, block, instruction, Some(result), target.signature.result)
}

fn validate_call_value(
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
    let definition = program.ty(callee_ty).ok_or_else(|| {
        unknown(
            function,
            block,
            instruction,
            EntityKind::Type,
            callee_ty.get(),
        )
    })?;
    let (parameters, expected_result) =
        function_value_signature(function, block, instruction, callee_ty, Some(&definition.kind))?;
    validate_arguments(scope, arguments, parameters)?;
    if let Some(result) = result
        && !types_compatible(program, expected_result, result.ty)
    {
        require_result_type(function, block, instruction, Some(result), expected_result)?;
    }
    Ok(())
}

/// Return the parameter and result types of a function-typed callee value.
fn function_value_signature<'p>(
    function: &Function,
    block: BlockId,
    instruction: usize,
    callee_ty: TypeId,
    kind: Option<&'p IrType>,
) -> Result<(&'p [TypeId], TypeId), ValidationError> {
    match kind {
        Some(IrType::Function { parameters, result }) => Ok((parameters, *result)),
        _ => Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::CallValueType {
                actual: callee_ty.get(),
            },
        )),
    }
}

fn validate_arguments(
    scope: OperandScope<'_>,
    arguments: &[ValueId],
    parameters: &[TypeId],
) -> Result<(), ValidationError> {
    let OperandScope {
        program,
        function,
        block,
        instruction,
        all_values,
        available,
    } = scope;
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
        if !types_compatible(program, *expected, actual) {
            require_exact(function, block, instruction, "call argument", *expected, actual)?;
        }
    }
    Ok(())
}

fn types_compatible(program: &Program, expected: TypeId, actual: TypeId) -> bool {
    if expected == actual {
        return true;
    }
    match (
        program.ty(expected).map(|item| &item.kind),
        program.ty(actual).map(|item| &item.kind),
    ) {
        (Some(IrType::Dynamic), _) | (_, Some(IrType::Dynamic)) => true,
        (Some(IrType::Array(a)), Some(IrType::Array(b)))
        | (Some(IrType::Option(a)), Some(IrType::Option(b)))
        | (Some(IrType::Task(a)), Some(IrType::Task(b)))
        | (Some(IrType::Channel(a)), Some(IrType::Channel(b))) => {
            types_compatible(program, *a, *b)
        }
        (
            Some(IrType::Dictionary { key: ak, value: av }),
            Some(IrType::Dictionary { key: bk, value: bv }),
        ) => types_compatible(program, *ak, *bk) && types_compatible(program, *av, *bv),
        (
            Some(IrType::Result {
                ok: ao,
                error: ae,
            }),
            Some(IrType::Result {
                ok: bo,
                error: be,
            }),
        ) => types_compatible(program, *ao, *bo) && types_compatible(program, *ae, *be),
        (
            Some(IrType::Function {
                parameters: ap,
                result: ar,
            }),
            Some(IrType::Function {
                parameters: bp,
                result: br,
            }),
        ) => {
            ap.len() == bp.len()
                && ap
                    .iter()
                    .zip(bp)
                    .all(|(a, b)| types_compatible(program, *a, *b))
                && types_compatible(program, *ar, *br)
        }
        (Some(IrType::Record(a)), Some(IrType::Record(b))) => a == b,
        (Some(IrType::Enum(a)), Some(IrType::Enum(b))) => a == b,
        _ => false,
    }
}

fn validate_intrinsic(
    scope: OperandScope<'_>,
    intrinsic: crate::IntrinsicId,
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
    let intrinsic = program.intrinsic(intrinsic).ok_or_else(|| {
        unknown(
            function,
            block,
            instruction,
            EntityKind::Intrinsic,
            intrinsic.get(),
        )
    })?;
    if intrinsic.variadic {
        validate_variadic_arguments(scope, arguments, &intrinsic.parameters)?;
    } else {
        validate_arguments(scope, arguments, &intrinsic.parameters)?;
    }
    let result = result.ok_or_else(|| {
        function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::MissingResult,
        )
    })?;
    require_assignable(
        program,
        function,
        block,
        instruction,
        "result",
        intrinsic.result,
        result.ty,
    )
}

fn validate_variadic_arguments(
    scope: OperandScope<'_>,
    arguments: &[ValueId],
    parameters: &[TypeId],
) -> Result<(), ValidationError> {
    if arguments.len() < parameters.len() || parameters.is_empty() {
        return validate_arguments(scope, arguments, parameters);
    }
    let repeated = parameters[parameters.len() - 1];
    let expected = parameters
        .iter()
        .copied()
        .chain(std::iter::repeat(repeated))
        .take(arguments.len())
        .collect::<Vec<_>>();
    validate_arguments(scope, arguments, &expected)
}

fn validate_closure(
    scope: OperandScope<'_>,
    target: FunctionId,
    captures: &[ValueId],
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
    let target = program.function(target).ok_or_else(|| {
        unknown(
            function,
            block,
            instruction,
            EntityKind::Function,
            target.get(),
        )
    })?;
    if captures.len() != target.captures.len() {
        return Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::ClosureCaptureCount {
                expected: target.captures.len(),
                actual: captures.len(),
            },
        ));
    }
    for (index, (capture, declaration)) in captures.iter().zip(&target.captures).enumerate() {
        let actual = value_type(
            function,
            block,
            instruction,
            *capture,
            all_values,
            available,
        )?;
        let actual_capture_type = match declaration.kind {
            crate::CaptureKind::Value => actual,
            crate::CaptureKind::Cell | crate::CaptureKind::EnclosingCell => {
                let Some(IrType::Cell(inner)) =
                    program.ty(actual).map(|definition| &definition.kind)
                else {
                    return Err(function_error(
                        function.id,
                        Some(block),
                        Some(instruction),
                        ValidationErrorKind::ClosureCaptureType {
                            index,
                            expected: declaration.ty.get(),
                            actual: actual.get(),
                        },
                    ));
                };
                *inner
            }
        };
        if actual_capture_type != declaration.ty {
            return Err(function_error(
                function.id,
                Some(block),
                Some(instruction),
                ValidationErrorKind::ClosureCaptureType {
                    index,
                    expected: declaration.ty.get(),
                    actual: actual_capture_type.get(),
                },
            ));
        }
    }
    let result = result.ok_or_else(|| {
        function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::MissingResult,
        )
    })?;
    match program.ty(result.ty).map(|definition| &definition.kind) {
        Some(IrType::Function {
            parameters,
            result: output,
        }) if parameters == &target.signature.parameters && *output == target.signature.result => {
            Ok(())
        }
        _ => Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::OperandType {
                operand: "closure result",
                expected: target.signature.result.get(),
                actual: result.ty.get(),
            },
        )),
    }
}

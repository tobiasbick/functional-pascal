#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_intrinsic(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    intrinsic: crate::IntrinsicId,
    arguments: &[ValueId],
    result: Option<ValueDefinition>,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
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
        validate_variadic_arguments(
            program,
            function,
            block,
            instruction,
            arguments,
            &intrinsic.parameters,
            all_values,
            available,
        )?;
    } else {
        validate_arguments(
            program,
            function,
            block,
            instruction,
            arguments,
            &intrinsic.parameters,
            all_values,
            available,
        )?;
    }
    let result = result.ok_or_else(|| {
        function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::MissingResult,
        )
    })?;
    if types_compatible(program, intrinsic.result, result.ty) {
        Ok(())
    } else {
        require_exact(
            function,
            block,
            instruction,
            "result",
            intrinsic.result,
            result.ty,
        )
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_variadic_arguments(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    arguments: &[ValueId],
    parameters: &[TypeId],
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
    if arguments.len() < parameters.len() || parameters.is_empty() {
        return validate_arguments(
            program,
            function,
            block,
            instruction,
            arguments,
            parameters,
            all_values,
            available,
        );
    }
    let repeated = parameters[parameters.len() - 1];
    let expected = parameters
        .iter()
        .copied()
        .chain(std::iter::repeat(repeated))
        .take(arguments.len())
        .collect::<Vec<_>>();
    validate_arguments(
        program,
        function,
        block,
        instruction,
        arguments,
        &expected,
        all_values,
        available,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_closure(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    target: FunctionId,
    captures: &[ValueId],
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

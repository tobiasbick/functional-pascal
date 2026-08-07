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
    require_result_type(function, block, instruction, result, intrinsic.result)
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
            ValidationErrorKind::ClosureCapture {
                index: captures.len(),
                expected: target.captures.len() as u32,
                actual: captures.len() as u32,
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
                        ValidationErrorKind::ClosureCapture {
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
                ValidationErrorKind::ClosureCapture {
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

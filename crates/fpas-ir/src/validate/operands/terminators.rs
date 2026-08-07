fn validate_terminator(
    program: &Program,
    function: &Function,
    block: BlockId,
    terminator: &Terminator,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
    match terminator {
        Terminator::Branch {
            condition,
            then_target,
            else_target,
        } => {
            let condition = value_type(function, block, None, *condition, all_values, available)?;
            require_category(
                program,
                function,
                block,
                None,
                "branch condition",
                condition,
                TypeCategory::Boolean,
            )?;
            validate_target(program, function, block, then_target, all_values, available)?;
            validate_target(program, function, block, else_target, all_values, available)
        }
        Terminator::Jump(target) => {
            validate_target(program, function, block, target, all_values, available)
        }
        Terminator::Return(value) => match value {
            Some(value) => {
                let actual = value_type(function, block, None, *value, all_values, available)?;
                require_exact(
                    function,
                    block,
                    None,
                    "return value",
                    function.signature.result,
                    actual,
                )
            }
            None => require_category(
                program,
                function,
                block,
                None,
                "return value",
                function.signature.result,
                TypeCategory::Unit,
            ),
        },
        Terminator::Panic(value) => {
            value_type(function, block, None, *value, all_values, available).map(drop)
        }
    }
}

fn validate_target(
    program: &Program,
    function: &Function,
    block: BlockId,
    target: &crate::BlockTarget,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
    let target_block = function
        .block(target.block)
        .ok_or_else(|| unknown(function, block, None, EntityKind::Block, target.block.get()))?;
    if target.arguments.len() != target_block.parameters.len() {
        return Err(function_error(
            function.id,
            Some(block),
            None,
            ValidationErrorKind::BlockArgumentCount {
                expected: target_block.parameters.len(),
                actual: target.arguments.len(),
            },
        ));
    }
    for (argument, parameter) in target.arguments.iter().zip(&target_block.parameters) {
        let actual = value_type(function, block, None, *argument, all_values, available)?;
        if actual != parameter.ty {
            return Err(function_error(
                function.id,
                Some(block),
                None,
                ValidationErrorKind::BlockArgumentType {
                    expected: parameter.ty.get(),
                    actual: actual.get(),
                },
            ));
        }
    }
    let _ = program;
    Ok(())
}

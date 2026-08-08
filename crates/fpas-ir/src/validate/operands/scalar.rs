fn validate_constant(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    constant: &Constant,
    result: Option<ValueDefinition>,
) -> Result<(), ValidationError> {
    let category = match constant {
        Constant::Unit => TypeCategory::Unit,
        Constant::Boolean(_) => TypeCategory::Boolean,
        Constant::Integer(_) => TypeCategory::Integer,
        Constant::Real(_) => TypeCategory::Real,
        Constant::String(_) => TypeCategory::String,
    };
    require_result_category(program, function, block, instruction, result, category)
}

#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_binary(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    operation: BinaryOperation,
    left: TypeId,
    right: TypeId,
    result: Option<ValueDefinition>,
) -> Result<(), ValidationError> {
    let (operands, output) = binary_categories(operation);
    if operands == TypeCategory::Same {
        if !types_compatible(program, left, right) {
            require_exact(function, block, instruction, "right operand", left, right)?;
        }
    } else {
        require_category(
            program,
            function,
            block,
            instruction,
            "left operand",
            left,
            operands,
        )?;
        require_category(
            program,
            function,
            block,
            instruction,
            "right operand",
            right,
            operands,
        )?;
    }
    require_result_category(program, function, block, instruction, result, output)
}

fn validate_unary(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    operation: crate::UnaryOperation,
    operand: TypeId,
    result: Option<ValueDefinition>,
) -> Result<(), ValidationError> {
    let (input, output) = unary_categories(operation);
    require_category(
        program,
        function,
        block,
        instruction,
        "unary operand",
        operand,
        input,
    )?;
    require_result_category(program, function, block, instruction, result, output)
}

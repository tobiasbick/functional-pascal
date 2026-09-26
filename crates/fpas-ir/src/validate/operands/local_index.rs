fn validate_store_local_index(
    scope: OperandScope<'_>,
    local: LocalId,
    index: ValueId,
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
    let local = function
        .local(local)
        .ok_or_else(|| unknown(function, block, instruction, EntityKind::Local, local.get()))?;
    if !local.mutable {
        return invalid_p5_result(function, block, instruction, local.ty);
    }
    let index_ty = value_type(function, block, instruction, index, all_values, available)?;
    let value_ty = value_type(function, block, instruction, value, all_values, available)?;
    let Some(expected) =
        indexed_element_type(program, function, block, instruction, local.ty, index_ty)?
    else {
        return invalid_p5_result(function, block, instruction, local.ty);
    };
    require_assignable(
        program,
        function,
        block,
        instruction,
        "indexed value",
        expected,
        value_ty,
    )
}

fn validate_store_global_index_path(
    scope: OperandScope<'_>,
    global: crate::GlobalId,
    root: ValueId,
    indexes: &[ValueId],
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
    let global = program.global(global).ok_or_else(|| {
        unknown(
            function,
            block,
            instruction,
            EntityKind::Global,
            global.get(),
        )
    })?;
    let root_ty = value_type(
        function,
        block,
        instruction,
        root,
        all_values,
        available,
    )?;
    require_exact(
        function,
        block,
        instruction,
        "global snapshot",
        global.ty,
        root_ty,
    )?;

    let mut aggregate_ty = root_ty;
    for index in indexes {
        let index_ty = value_type(
            function,
            block,
            instruction,
            *index,
            all_values,
            available,
        )?;
        let Some(element) =
            indexed_element_type(program, function, block, instruction, aggregate_ty, index_ty)?
        else {
            return Err(function_error(
                function.id,
                Some(block),
                Some(instruction),
                ValidationErrorKind::OperandType {
                    operand: "indexed global aggregate",
                    expected: global.ty.get(),
                    actual: aggregate_ty.get(),
                },
            ));
        };
        aggregate_ty = element;
    }

    let value_ty = value_type(
        function,
        block,
        instruction,
        value,
        all_values,
        available,
    )?;
    require_assignable(
        program,
        function,
        block,
        instruction,
        "indexed global value",
        aggregate_ty,
        value_ty,
    )
}

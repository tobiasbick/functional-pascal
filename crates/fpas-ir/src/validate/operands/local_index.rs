#[expect(
    clippy::too_many_arguments,
    reason = "typed validation keeps operand scopes explicit"
)]
fn validate_store_local_index(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    local: LocalId,
    index: ValueId,
    value: ValueId,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
    let local = function
        .local(local)
        .ok_or_else(|| unknown(function, block, instruction, EntityKind::Local, local.get()))?;
    if !local.mutable {
        return invalid_p5_result(function, block, instruction, local.ty);
    }
    let index_ty = value_type(function, block, instruction, index, all_values, available)?;
    let value_ty = value_type(function, block, instruction, value, all_values, available)?;
    let expected = match program.ty(local.ty).map(|definition| &definition.kind) {
        Some(IrType::Array(element)) => {
            require_category(
                program,
                function,
                block,
                instruction,
                "array index",
                index_ty,
                TypeCategory::Integer,
            )?;
            *element
        }
        Some(IrType::Dictionary { key, value }) => {
            require_exact(
                function,
                block,
                instruction,
                "dictionary key",
                *key,
                index_ty,
            )?;
            *value
        }
        _ => return invalid_p5_result(function, block, instruction, local.ty),
    };
    if types_compatible(program, expected, value_ty) {
        return Ok(());
    }
    require_exact(
        function,
        block,
        instruction,
        "indexed value",
        expected,
        value_ty,
    )
}

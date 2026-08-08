#[expect(
    clippy::too_many_arguments,
    reason = "typed validation keeps operand scopes explicit"
)]
fn validate_store_global_index_path(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    global: crate::GlobalId,
    root: ValueId,
    indexes: &[ValueId],
    value: ValueId,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
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
        aggregate_ty = match program.ty(aggregate_ty).map(|definition| &definition.kind) {
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
            _ => {
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
            }
        };
    }

    let value_ty = value_type(
        function,
        block,
        instruction,
        value,
        all_values,
        available,
    )?;
    if types_compatible(program, aggregate_ty, value_ty) {
        return Ok(());
    }
    require_exact(
        function,
        block,
        instruction,
        "indexed global value",
        aggregate_ty,
        value_ty,
    )
}

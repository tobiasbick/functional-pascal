#[expect(
    clippy::too_many_arguments,
    reason = "typed validation keeps operand scopes explicit"
)]
fn validate_wrapper_test(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    value: ValueId,
    result: ValueDefinition,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
    result_wrapper: bool,
) -> Result<(), ValidationError> {
    let wrapper = value_type(function, block, instruction, value, all_values, available)?;
    let valid = match program.ty(wrapper).map(|definition| &definition.kind) {
        Some(IrType::Result { .. }) => result_wrapper,
        Some(IrType::Option(_)) => !result_wrapper,
        _ => false,
    };
    if !valid {
        return invalid_p5_result(function, block, instruction, wrapper);
    }
    require_result_category(
        program,
        function,
        block,
        instruction,
        Some(result),
        TypeCategory::Boolean,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "typed validation keeps operand scopes explicit"
)]
fn validate_wrapper(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    value: ValueId,
    result: ValueDefinition,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
    kind: u8,
) -> Result<(), ValidationError> {
    let payload = value_type(function, block, instruction, value, all_values, available)?;
    let expected = match (
        kind,
        program.ty(result.ty).map(|definition| &definition.kind),
    ) {
        (0, Some(IrType::Result { ok, .. })) => Some(*ok),
        (1, Some(IrType::Result { error, .. })) => Some(*error),
        (2, Some(IrType::Option(value))) => Some(*value),
        _ => None,
    };
    require_exact(
        function,
        block,
        instruction,
        "wrapper payload",
        expected.ok_or_else(|| {
            function_error(
                function.id,
                Some(block),
                Some(instruction),
                ValidationErrorKind::UnknownId {
                    entity: EntityKind::Type,
                    id: result.ty.get(),
                },
            )
        })?,
        payload,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "typed validation keeps operand scopes explicit"
)]
fn validate_unwrap(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    value: ValueId,
    result: ValueDefinition,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
    kind: u8,
) -> Result<(), ValidationError> {
    let wrapper = value_type(function, block, instruction, value, all_values, available)?;
    let expected = match (kind, program.ty(wrapper).map(|definition| &definition.kind)) {
        (0, Some(IrType::Result { ok, .. })) => Some(*ok),
        (1, Some(IrType::Result { error, .. })) => Some(*error),
        (2, Some(IrType::Option(value))) => Some(*value),
        _ => None,
    };
    require_result_type(
        function,
        block,
        instruction,
        Some(result),
        expected.ok_or_else(|| {
            function_error(
                function.id,
                Some(block),
                Some(instruction),
                ValidationErrorKind::UnknownId {
                    entity: EntityKind::Type,
                    id: wrapper.get(),
                },
            )
        })?,
    )
}

fn invalid_p5_result(
    function: &Function,
    block: BlockId,
    instruction: usize,
    ty: TypeId,
) -> Result<(), ValidationError> {
    Err(function_error(
        function.id,
        Some(block),
        Some(instruction),
        ValidationErrorKind::UnknownId {
            entity: EntityKind::Type,
            id: ty.get(),
        },
    ))
}

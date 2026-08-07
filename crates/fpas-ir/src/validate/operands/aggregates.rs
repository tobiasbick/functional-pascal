#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_record_make(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    layout: RecordLayoutId,
    fields: &[ValueId],
    result: Option<ValueDefinition>,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
    let layout = program.record_layout(layout).ok_or_else(|| {
        unknown(
            function,
            block,
            instruction,
            EntityKind::RecordLayout,
            layout.get(),
        )
    })?;
    validate_arguments(
        program,
        function,
        block,
        instruction,
        fields,
        &layout
            .fields
            .iter()
            .map(|field| field.ty)
            .collect::<Vec<_>>(),
        all_values,
        available,
    )?;
    let result = result.ok_or_else(|| {
        function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::MissingResult,
        )
    })?;
    match program.ty(result.ty).map(|definition| &definition.kind) {
        Some(IrType::Record(actual)) if *actual == layout.id => Ok(()),
        _ => Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::LayoutReference {
                expected: layout.id.get(),
                actual: result.ty.get(),
            },
        )),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_field_load(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    record: ValueId,
    layout: RecordLayoutId,
    field: crate::FieldId,
    result: Option<ValueDefinition>,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
    let record_ty = value_type(function, block, instruction, record, all_values, available)?;
    require_record_layout(program, function, block, instruction, record_ty, layout)?;
    let field = record_field(program, function, block, instruction, layout, field)?;
    require_result_type(function, block, instruction, result, field.ty)
}

#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_field_store(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    record: ValueId,
    layout: RecordLayoutId,
    field: crate::FieldId,
    value: ValueId,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
    let record_ty = value_type(function, block, instruction, record, all_values, available)?;
    require_record_layout(program, function, block, instruction, record_ty, layout)?;
    let field = record_field(program, function, block, instruction, layout, field)?;
    let value_ty = value_type(function, block, instruction, value, all_values, available)?;
    require_exact(
        function,
        block,
        instruction,
        "field value",
        field.ty,
        value_ty,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_enum_make(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    layout: EnumLayoutId,
    variant: crate::VariantId,
    fields: &[ValueId],
    result: Option<ValueDefinition>,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
    let layout = program.enum_layout(layout).ok_or_else(|| {
        unknown(
            function,
            block,
            instruction,
            EntityKind::EnumLayout,
            layout.get(),
        )
    })?;
    let variant = layout
        .variants
        .iter()
        .find(|item| item.id == variant)
        .ok_or_else(|| {
            unknown(
                function,
                block,
                instruction,
                EntityKind::Variant,
                variant.get(),
            )
        })?;
    validate_arguments(
        program,
        function,
        block,
        instruction,
        fields,
        &variant.fields,
        all_values,
        available,
    )?;
    let result = result.ok_or_else(|| {
        function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::MissingResult,
        )
    })?;
    match program.ty(result.ty).map(|definition| &definition.kind) {
        Some(IrType::Enum(actual)) if *actual == layout.id => Ok(()),
        _ => Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::LayoutReference {
                expected: layout.id.get(),
                actual: result.ty.get(),
            },
        )),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_variant_test(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    value: ValueId,
    layout: EnumLayoutId,
    variant: crate::VariantId,
    result: Option<ValueDefinition>,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
    let value_ty = value_type(function, block, instruction, value, all_values, available)?;
    require_enum_layout(program, function, block, instruction, value_ty, layout)?;
    let enum_layout = program.enum_layout(layout).ok_or_else(|| {
        unknown(
            function,
            block,
            instruction,
            EntityKind::EnumLayout,
            layout.get(),
        )
    })?;
    if !enum_layout.variants.iter().any(|item| item.id == variant) {
        return Err(unknown(
            function,
            block,
            instruction,
            EntityKind::Variant,
            variant.get(),
        ));
    }
    require_result_category(
        program,
        function,
        block,
        instruction,
        result,
        TypeCategory::Boolean,
    )
}

fn validate_record_make(
    scope: OperandScope<'_>,
    layout: RecordLayoutId,
    fields: &[ValueId],
    result: Option<ValueDefinition>,
) -> Result<(), ValidationError> {
    let OperandScope {
        program,
        function,
        block,
        instruction,
        ..
    } = scope;
    let layout = program.record_layout(layout).ok_or_else(|| {
        unknown(
            function,
            block,
            instruction,
            EntityKind::RecordLayout,
            layout.get(),
        )
    })?;
    validate_arguments(scope, fields, &layout
            .fields
            .iter()
            .map(|field| field.ty)
            .collect::<Vec<_>>())?;
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

fn validate_field_load(
    scope: OperandScope<'_>,
    record: ValueId,
    layout: RecordLayoutId,
    field: crate::FieldId,
    result: Option<ValueDefinition>,
) -> Result<(), ValidationError> {
    let OperandScope {
        program,
        function,
        block,
        instruction,
        all_values,
        available,
    } = scope;
    let record_ty = value_type(function, block, instruction, record, all_values, available)?;
    require_record_layout(program, function, block, instruction, record_ty, layout)?;
    let field = record_field(program, function, block, instruction, layout, field)?;
    require_result_type(function, block, instruction, result, field.ty)
}

fn validate_field_store(
    scope: OperandScope<'_>,
    record: ValueId,
    layout: RecordLayoutId,
    field: crate::FieldId,
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

fn validate_enum_make(
    scope: OperandScope<'_>,
    layout: EnumLayoutId,
    variant: crate::VariantId,
    fields: &[ValueId],
    result: Option<ValueDefinition>,
) -> Result<(), ValidationError> {
    let OperandScope {
        program,
        function,
        block,
        instruction,
        ..
    } = scope;
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
    validate_arguments(scope, fields, &variant.fields)?;
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

fn validate_variant_test(
    scope: OperandScope<'_>,
    value: ValueId,
    layout: EnumLayoutId,
    variant: crate::VariantId,
    result: Option<ValueDefinition>,
) -> Result<(), ValidationError> {
    let OperandScope {
        program,
        function,
        block,
        instruction,
        all_values,
        available,
    } = scope;
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

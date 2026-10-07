// Reference creation, narrowing, reads, and writes for `var` parameters.
// Documentation: docs/pascal/language/functions/var-parameters.md

fn reference_target(
    scope: OperandScope<'_>,
    reference: ValueId,
) -> Result<TypeId, ValidationError> {
    let OperandScope {
        program,
        function,
        block,
        instruction,
        all_values,
        available,
    } = scope;
    let reference_ty = value_type(function, block, instruction, reference, all_values, available)?;
    match program.ty(reference_ty).map(|definition| &definition.kind) {
        Some(IrType::Reference(target)) => Ok(*target),
        _ => Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::TypeCategory {
                operand: "reference",
                expected: "reference",
                actual: reference_ty.get(),
            },
        )),
    }
}

fn require_reference_result(
    scope: OperandScope<'_>,
    result: Option<ValueDefinition>,
    target: TypeId,
) -> Result<(), ValidationError> {
    let OperandScope {
        program,
        function,
        block,
        instruction,
        ..
    } = scope;
    let result = result.ok_or_else(|| {
        function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::MissingResult,
        )
    })?;
    match program.ty(result.ty).map(|definition| &definition.kind) {
        Some(IrType::Reference(actual)) if *actual == target => Ok(()),
        _ => Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::OperandType {
                operand: "reference result",
                expected: target.get(),
                actual: result.ty.get(),
            },
        )),
    }
}

fn validate_cell_reference(
    scope: OperandScope<'_>,
    cell: ValueId,
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
    let cell_ty = value_type(function, block, instruction, cell, all_values, available)?;
    let Some(IrType::Cell(inner)) = program.ty(cell_ty).map(|definition| &definition.kind) else {
        return Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::TypeCategory {
                operand: "cell",
                expected: "cell",
                actual: cell_ty.get(),
            },
        ));
    };
    require_reference_result(scope, result, *inner)
}

fn validate_global_reference(
    scope: OperandScope<'_>,
    global: crate::GlobalId,
    result: Option<ValueDefinition>,
) -> Result<(), ValidationError> {
    let OperandScope {
        program,
        function,
        block,
        instruction,
        ..
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
    if !global.mutable {
        return Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::TypeCategory {
                operand: "global reference",
                expected: "mutable global",
                actual: global.ty.get(),
            },
        ));
    }
    require_reference_result(scope, result, global.ty)
}

fn validate_reference_field(
    scope: OperandScope<'_>,
    reference: ValueId,
    layout: RecordLayoutId,
    field: crate::FieldId,
    result: Option<ValueDefinition>,
) -> Result<(), ValidationError> {
    let OperandScope {
        program,
        function,
        block,
        instruction,
        ..
    } = scope;
    let target = reference_target(scope, reference)?;
    require_record_layout(program, function, block, instruction, target, layout)?;
    let field = record_field(program, function, block, instruction, layout, field)?;
    require_reference_result(scope, result, field.ty)
}

fn validate_reference_element(
    scope: OperandScope<'_>,
    reference: ValueId,
    index: ValueId,
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
    let target = reference_target(scope, reference)?;
    let Some(IrType::Array(element)) = program.ty(target).map(|definition| &definition.kind) else {
        return Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::TypeCategory {
                operand: "element reference",
                expected: "array reference",
                actual: target.get(),
            },
        ));
    };
    let index_ty = value_type(function, block, instruction, index, all_values, available)?;
    require_category(
        program,
        function,
        block,
        instruction,
        "element index",
        index_ty,
        TypeCategory::Integer,
    )?;
    require_reference_result(scope, result, *element)
}

fn validate_reference_read(
    scope: OperandScope<'_>,
    reference: ValueId,
    result: Option<ValueDefinition>,
) -> Result<(), ValidationError> {
    let target = reference_target(scope, reference)?;
    require_result_type(scope.function, scope.block, scope.instruction, result, target)
}

fn validate_reference_write(
    scope: OperandScope<'_>,
    reference: ValueId,
    value: ValueId,
) -> Result<(), ValidationError> {
    let OperandScope {
        function,
        block,
        instruction,
        all_values,
        available,
        ..
    } = scope;
    let target = reference_target(scope, reference)?;
    let value_ty = value_type(function, block, instruction, value, all_values, available)?;
    require_exact(function, block, instruction, "referenced value", target, value_ty)
}

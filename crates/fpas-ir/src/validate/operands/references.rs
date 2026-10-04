// Synchronous references retain the selected value type through every projection.
fn validate_reference_operation(
    scope: OperandScope<'_>,
    operation: &crate::ReferenceOperation,
    result: Option<ValueDefinition>,
) -> Result<(), ValidationError> {
    use crate::ReferenceOperation;
    let type_of = |value| {
        value_type(
            scope.function,
            scope.block,
            scope.instruction,
            value,
            scope.all_values,
            scope.available,
        )
    };
    match operation {
        ReferenceOperation::Reserve(root) => {
            let ty = type_of(*root)?;
            let inner = match scope.program.ty(ty).map(|definition| &definition.kind) {
                Some(IrType::Cell(inner) | IrType::Reference(inner)) => *inner,
                _ => {
                    return Err(reference_type_error(
                        scope,
                        "reference root",
                        "cell or reference",
                        ty,
                    ));
                }
            };
            require_reference_result(scope, result, inner)
        }
        ReferenceOperation::Field {
            reference,
            layout,
            field,
        } => {
            let inner = reference_value_type(scope, type_of(*reference)?)?;
            require_record_layout(
                scope.program,
                scope.function,
                scope.block,
                scope.instruction,
                inner,
                *layout,
            )?;
            let ty = record_field(
                scope.program,
                scope.function,
                scope.block,
                scope.instruction,
                *layout,
                *field,
            )?
            .ty;
            require_reference_result(scope, result, ty)
        }
        ReferenceOperation::Index { reference, index } => {
            let inner = reference_value_type(scope, type_of(*reference)?)?;
            let element = indexed_element_type(
                scope.program,
                scope.function,
                scope.block,
                scope.instruction,
                inner,
                type_of(*index)?,
            )?
            .ok_or_else(|| {
                reference_type_error(scope, "selected collection", "array or dictionary", inner)
            })?;
            require_reference_result(scope, result, element)
        }
        ReferenceOperation::Read(reference) => {
            let inner = reference_value_type(scope, type_of(*reference)?)?;
            require_result_type(
                scope.function,
                scope.block,
                scope.instruction,
                result,
                inner,
            )
        }
        ReferenceOperation::Write { reference, value } => {
            let inner = reference_value_type(scope, type_of(*reference)?)?;
            require_exact(
                scope.function,
                scope.block,
                scope.instruction,
                "reference replacement",
                inner,
                type_of(*value)?,
            )
        }
        ReferenceOperation::Release(reference) => {
            reference_value_type(scope, type_of(*reference)?).map(|_| ())
        }
    }
}

fn reference_value_type(scope: OperandScope<'_>, ty: TypeId) -> Result<TypeId, ValidationError> {
    match scope.program.ty(ty).map(|definition| &definition.kind) {
        Some(IrType::Reference(inner)) => Ok(*inner),
        _ => Err(reference_type_error(
            scope,
            "selected storage",
            "reference",
            ty,
        )),
    }
}

fn require_reference_result(
    scope: OperandScope<'_>,
    result: Option<ValueDefinition>,
    inner: TypeId,
) -> Result<(), ValidationError> {
    let result = result.ok_or_else(|| {
        function_error(
            scope.function.id,
            Some(scope.block),
            Some(scope.instruction),
            ValidationErrorKind::MissingResult,
        )
    })?;
    match scope
        .program
        .ty(result.ty)
        .map(|definition| &definition.kind)
    {
        Some(IrType::Reference(actual)) if *actual == inner => Ok(()),
        _ => Err(reference_type_error(
            scope,
            "reference result",
            "reference to selected value type",
            result.ty,
        )),
    }
}

fn reference_type_error(
    scope: OperandScope<'_>,
    operand: &'static str,
    expected: &'static str,
    ty: TypeId,
) -> ValidationError {
    function_error(
        scope.function.id,
        Some(scope.block),
        Some(scope.instruction),
        ValidationErrorKind::TypeCategory {
            operand,
            expected,
            actual: ty.get(),
        },
    )
}

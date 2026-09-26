fn validate_p5(
    scope: OperandScope<'_>,
    operation: &Operation,
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
    let result = result.ok_or_else(|| {
        function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::MissingResult,
        )
    })?;
    match operation {
        Operation::MakeArray(values) => {
            match program.ty(result.ty).map(|definition| &definition.kind) {
                Some(IrType::Array(element)) => validate_arguments(scope, values, &vec![*element; values.len()]),
                _ => invalid_p5_result(function, block, instruction, result.ty),
            }
        }
        Operation::ArrayPush { local, value } => {
            let local = function.local(*local).ok_or_else(|| {
                unknown(
                    function,
                    block,
                    instruction,
                    EntityKind::Local,
                    local.get(),
                )
            })?;
            if !local.mutable {
                return invalid_p5_result(function, block, instruction, local.ty);
            }
            let value_ty = value_type(
                function,
                block,
                instruction,
                *value,
                all_values,
                available,
            )?;
            let Some(IrType::Array(element)) =
                program.ty(local.ty).map(|definition| &definition.kind)
            else {
                return invalid_p5_result(function, block, instruction, local.ty);
            };
            if !types_compatible(program, *element, value_ty) {
                require_exact(
                    function,
                    block,
                    instruction,
                    "array element",
                    *element,
                    value_ty,
                )?;
            }
            require_result_category(
                program,
                function,
                block,
                instruction,
                Some(result),
                TypeCategory::Unit,
            )
        }
        Operation::ArrayPop { local } => {
            let local = function.local(*local).ok_or_else(|| {
                unknown(function, block, instruction, EntityKind::Local, local.get())
            })?;
            if !local.mutable {
                return invalid_p5_result(function, block, instruction, local.ty);
            }
            let Some(IrType::Array(element)) = program.ty(local.ty).map(|definition| &definition.kind) else {
                return invalid_p5_result(function, block, instruction, local.ty);
            };
            require_exact(function, block, instruction, "popped array element", *element, result.ty)
        }
        Operation::MakeDictionary(pairs) => {
            match program.ty(result.ty).map(|definition| &definition.kind) {
                Some(IrType::Dictionary { key, value }) => {
                    let values = pairs
                        .iter()
                        .flat_map(|(key, value)| [*key, *value])
                        .collect::<Vec<_>>();
                    let expected = pairs
                        .iter()
                        .flat_map(|_| [*key, *value])
                        .collect::<Vec<_>>();
                    validate_arguments(scope, &values, &expected)
                }
                _ => invalid_p5_result(function, block, instruction, result.ty),
            }
        }
        Operation::IndexGet { collection, index } => {
            let collection_ty = value_type(
                function,
                block,
                instruction,
                *collection,
                all_values,
                available,
            )?;
            let index_ty = value_type(function, block, instruction, *index, all_values, available)?;
            if let Some(element) = indexed_element_type(
                program,
                function,
                block,
                instruction,
                collection_ty,
                index_ty,
            )? {
                return require_result_type(function, block, instruction, Some(result), element);
            }
            match program.ty(collection_ty).map(|definition| &definition.kind) {
                Some(IrType::String) => {
                    require_category(
                        program,
                        function,
                        block,
                        instruction,
                        "string index",
                        index_ty,
                        TypeCategory::Integer,
                    )?;
                    require_result_category(
                        program,
                        function,
                        block,
                        instruction,
                        Some(result),
                        TypeCategory::String,
                    )
                }
                _ => invalid_p5_result(function, block, instruction, collection_ty),
            }
        }
        Operation::IndexSet {
            collection,
            index,
            value,
        } => {
            let collection_ty = value_type(
                function,
                block,
                instruction,
                *collection,
                all_values,
                available,
            )?;
            let index_ty = value_type(function, block, instruction, *index, all_values, available)?;
            let value_ty = value_type(function, block, instruction, *value, all_values, available)?;
            let Some(expected) = indexed_element_type(
                program,
                function,
                block,
                instruction,
                collection_ty,
                index_ty,
            )?
            else {
                return invalid_p5_result(function, block, instruction, collection_ty);
            };
            require_assignable(
                program,
                function,
                block,
                instruction,
                "indexed value",
                expected,
                value_ty,
            )?;
            require_result_type(function, block, instruction, Some(result), collection_ty)
        }
        Operation::Contains { value, collection } => {
            let collection_ty = value_type(
                function,
                block,
                instruction,
                *collection,
                all_values,
                available,
            )?;
            let value_ty = value_type(function, block, instruction, *value, all_values, available)?;
            match program.ty(collection_ty).map(|definition| &definition.kind) {
                Some(IrType::Array(element)) => require_exact(
                    function,
                    block,
                    instruction,
                    "membership value",
                    *element,
                    value_ty,
                )?,
                Some(IrType::Dictionary { key, .. }) => require_exact(
                    function,
                    block,
                    instruction,
                    "membership value",
                    *key,
                    value_ty,
                )?,
                Some(IrType::String) => require_category(
                    program,
                    function,
                    block,
                    instruction,
                    "membership value",
                    value_ty,
                    TypeCategory::String,
                )?,
                _ => return invalid_p5_result(function, block, instruction, collection_ty),
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
        Operation::UpdateRecord {
            record,
            layout,
            fields,
        } => {
            let record_ty =
                value_type(function, block, instruction, *record, all_values, available)?;
            require_record_layout(program, function, block, instruction, record_ty, *layout)?;
            for (field, value) in fields {
                let expected =
                    record_field(program, function, block, instruction, *layout, *field)?.ty;
                require_exact(
                    function,
                    block,
                    instruction,
                    "record override",
                    expected,
                    value_type(function, block, instruction, *value, all_values, available)?,
                )?;
            }
            require_result_type(function, block, instruction, Some(result), record_ty)
        }
        Operation::MakeOk(value) => validate_wrapper(scope, *value, result, 0),
        Operation::MakeError(value) => validate_wrapper(scope, *value, result, 1),
        Operation::MakeSome(value) => validate_wrapper(scope, *value, result, 2),
        Operation::MakeNone => {
            if matches!(
                program.ty(result.ty).map(|definition| &definition.kind),
                Some(IrType::Option(_))
            ) {
                Ok(())
            } else {
                invalid_p5_result(function, block, instruction, result.ty)
            }
        }
        Operation::IsResultOk(value) => validate_wrapper_test(scope, *value, result, true),
        Operation::IsOptionSome(value) => validate_wrapper_test(scope, *value, result, false),
        Operation::UnwrapOk(value) => validate_unwrap(scope, *value, result, 0),
        Operation::UnwrapError(value) => validate_unwrap(scope, *value, result, 1),
        Operation::UnwrapSome(value) => validate_unwrap(scope, *value, result, 2),
        Operation::LoadEnumField {
            value,
            layout,
            variant,
            field,
        } => {
            require_enum_layout(
                program,
                function,
                block,
                instruction,
                value_type(function, block, instruction, *value, all_values, available)?,
                *layout,
            )?;
            let variant = program
                .enum_layout(*layout)
                .and_then(|layout| layout.variants.iter().find(|item| item.id == *variant))
                .ok_or_else(|| {
                    unknown(
                        function,
                        block,
                        instruction,
                        EntityKind::Variant,
                        variant.get(),
                    )
                })?;
            let field_index = usize::try_from(field.get()).map_err(|_| {
                unknown(function, block, instruction, EntityKind::Field, field.get())
            })?;
            let ty = variant.fields.get(field_index).copied().ok_or_else(|| {
                unknown(function, block, instruction, EntityKind::Field, field.get())
            })?;
            require_result_type(function, block, instruction, Some(result), ty)
        }
        _ => invalid_p5_result(function, block, instruction, result.ty),
    }
}

include!("p5/wrappers.rs");

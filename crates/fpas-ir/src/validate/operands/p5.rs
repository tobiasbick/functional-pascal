#[expect(
    clippy::too_many_arguments,
    reason = "typed validation keeps operand scopes explicit"
)]
fn validate_p5(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    operation: &Operation,
    result: Option<ValueDefinition>,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
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
                Some(IrType::Array(element)) => validate_arguments(
                    program,
                    function,
                    block,
                    instruction,
                    values,
                    &vec![*element; values.len()],
                    all_values,
                    available,
                ),
                _ => invalid_p5_result(function, block, instruction, result.ty),
            }
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
                    validate_arguments(
                        program,
                        function,
                        block,
                        instruction,
                        &values,
                        &expected,
                        all_values,
                        available,
                    )
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
            match program.ty(collection_ty).map(|definition| &definition.kind) {
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
                    require_result_type(function, block, instruction, Some(result), *element)
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
                    require_result_type(function, block, instruction, Some(result), *value)
                }
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
            let expected = match program.ty(collection_ty).map(|definition| &definition.kind) {
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
                _ => return invalid_p5_result(function, block, instruction, collection_ty),
            };
            if !types_compatible(program, expected, value_ty) {
                require_exact(
                    function,
                    block,
                    instruction,
                    "indexed value",
                    expected,
                    value_ty,
                )?;
            }
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
        Operation::MakeOk(value) => validate_wrapper(
            program,
            function,
            block,
            instruction,
            *value,
            result,
            all_values,
            available,
            0,
        ),
        Operation::MakeError(value) => validate_wrapper(
            program,
            function,
            block,
            instruction,
            *value,
            result,
            all_values,
            available,
            1,
        ),
        Operation::MakeSome(value) => validate_wrapper(
            program,
            function,
            block,
            instruction,
            *value,
            result,
            all_values,
            available,
            2,
        ),
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
        Operation::IsResultOk(value) => validate_wrapper_test(
            program,
            function,
            block,
            instruction,
            *value,
            result,
            all_values,
            available,
            true,
        ),
        Operation::IsOptionSome(value) => validate_wrapper_test(
            program,
            function,
            block,
            instruction,
            *value,
            result,
            all_values,
            available,
            false,
        ),
        Operation::UnwrapOk(value) => validate_unwrap(
            program,
            function,
            block,
            instruction,
            *value,
            result,
            all_values,
            available,
            0,
        ),
        Operation::UnwrapError(value) => validate_unwrap(
            program,
            function,
            block,
            instruction,
            *value,
            result,
            all_values,
            available,
            1,
        ),
        Operation::UnwrapSome(value) => validate_unwrap(
            program,
            function,
            block,
            instruction,
            *value,
            result,
            all_values,
            available,
            2,
        ),
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

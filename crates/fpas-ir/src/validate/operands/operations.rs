fn validate_block(
    program: &Program,
    function: &Function,
    block: &BasicBlock,
    all_values: &BTreeMap<ValueId, TypeId>,
    parameter_values: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
    let mut available = parameter_values.clone();
    available.extend(block.parameters.iter().map(|parameter| parameter.id));
    for (instruction_index, instruction) in block.instructions.iter().enumerate() {
        validate_result_shape(function, block.id, instruction_index, instruction)?;
        validate_operation(
            program,
            function,
            block.id,
            instruction_index,
            &instruction.operation,
            instruction.result,
            all_values,
            &available,
        )?;
        if let Some(result) = instruction.result {
            available.insert(result.id);
        }
    }
    let Some(terminator) = block.terminators.first() else {
        return Ok(());
    };
    validate_terminator(
        program, function, block.id, terminator, all_values, &available,
    )
}

fn validate_result_shape(
    function: &Function,
    block: BlockId,
    instruction: usize,
    value: &crate::Instruction,
) -> Result<(), ValidationError> {
    match (value.operation.produces_value(), value.result.is_some()) {
        (true, false) => Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::MissingResult,
        )),
        (false, true) => Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::UnexpectedResult,
        )),
        _ => Ok(()),
    }
}

// Validation keeps ownership and source location explicit at each operation boundary.
#[expect(
    clippy::too_many_arguments,
    reason = "typed validation needs explicit operand scopes"
)]
fn validate_operation(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    operation: &Operation,
    result: Option<ValueDefinition>,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<(), ValidationError> {
    match operation {
        Operation::Const(constant) => {
            validate_constant(program, function, block, instruction, constant, result)
        }
        Operation::ReadLocal(local) => {
            let local = function.local(*local).ok_or_else(|| {
                unknown(function, block, instruction, EntityKind::Local, local.get())
            })?;
            require_result_type(function, block, instruction, result, local.ty)
        }
        Operation::WriteLocal { value, local } => {
            let value_ty = value_type(function, block, instruction, *value, all_values, available)?;
            let local = function.local(*local).ok_or_else(|| {
                unknown(function, block, instruction, EntityKind::Local, local.get())
            })?;
            if types_compatible(program, local.ty, value_ty) {
                return Ok(());
            }
            require_exact(
                function,
                block,
                instruction,
                "local value",
                local.ty,
                value_ty,
            )
        }
        Operation::Binary {
            operation,
            left,
            right,
        } => {
            let left_ty = value_type(function, block, instruction, *left, all_values, available)?;
            let right_ty = value_type(function, block, instruction, *right, all_values, available)?;
            validate_binary(
                program,
                function,
                block,
                instruction,
                *operation,
                left_ty,
                right_ty,
                result,
            )
        }
        Operation::Unary { operation, operand } => {
            let operand_ty =
                value_type(function, block, instruction, *operand, all_values, available)?;
            validate_unary(
                program,
                function,
                block,
                instruction,
                *operation,
                operand_ty,
                result,
            )
        }
        Operation::CallDirect {
            function: target,
            arguments,
        } => validate_direct_call(
            program,
            function,
            block,
            instruction,
            *target,
            arguments,
            result,
            all_values,
            available,
        ),
        Operation::CallValue { callee, arguments } => validate_call_value(
            program,
            function,
            block,
            instruction,
            *callee,
            arguments,
            result,
            all_values,
            available,
        ),
        Operation::LoadGlobal(global) => {
            let global = program.global(*global).ok_or_else(|| {
                unknown(
                    function,
                    block,
                    instruction,
                    EntityKind::Global,
                    global.get(),
                )
            })?;
            require_result_type(function, block, instruction, result, global.ty)
        }
        Operation::StoreGlobal { global, value } => {
            let global = program.global(*global).ok_or_else(|| {
                unknown(
                    function,
                    block,
                    instruction,
                    EntityKind::Global,
                    global.get(),
                )
            })?;
            let value_ty = value_type(function, block, instruction, *value, all_values, available)?;
            if types_compatible(program, global.ty, value_ty) {
                return Ok(());
            }
            require_exact(
                function,
                block,
                instruction,
                "global value",
                global.ty,
                value_ty,
            )
        }
        Operation::StoreGlobalIndexPath {
            global,
            root,
            indexes,
            value,
        } => validate_store_global_index_path(
            program,
            function,
            block,
            instruction,
            *global,
            *root,
            indexes,
            *value,
            all_values,
            available,
        ),
        Operation::MakeArray(_)
        | Operation::ArrayPush { .. }
        | Operation::MakeDictionary(_)
        | Operation::IndexGet { .. }
        | Operation::IndexSet { .. }
        | Operation::Contains { .. }
        | Operation::UpdateRecord { .. }
        | Operation::MakeOk(_)
        | Operation::MakeError(_)
        | Operation::MakeSome(_)
        | Operation::MakeNone
        | Operation::IsResultOk(_)
        | Operation::IsOptionSome(_)
        | Operation::UnwrapOk(_)
        | Operation::UnwrapError(_)
        | Operation::UnwrapSome(_)
        | Operation::LoadEnumField { .. } => validate_p5(
            program, function, block, instruction, operation, result, all_values, available,
        ),
        Operation::MakeRecord { layout, fields } => validate_record_make(
            program,
            function,
            block,
            instruction,
            *layout,
            fields,
            result,
            all_values,
            available,
        ),
        Operation::LoadField {
            record,
            layout,
            field,
        } => validate_field_load(
            program,
            function,
            block,
            instruction,
            *record,
            *layout,
            *field,
            result,
            all_values,
            available,
        ),
        Operation::StoreField {
            record,
            layout,
            field,
            value,
        } => validate_field_store(
            program,
            function,
            block,
            instruction,
            *record,
            *layout,
            *field,
            *value,
            all_values,
            available,
        ),
        Operation::MakeEnum {
            layout,
            variant,
            fields,
        } => validate_enum_make(
            program,
            function,
            block,
            instruction,
            *layout,
            *variant,
            fields,
            result,
            all_values,
            available,
        ),
        Operation::TestVariant {
            value,
            layout,
            variant,
        } => validate_variant_test(
            program,
            function,
            block,
            instruction,
            *value,
            *layout,
            *variant,
            result,
            all_values,
            available,
        ),
        Operation::Intrinsic {
            intrinsic,
            arguments,
        } => validate_intrinsic(
            program,
            function,
            block,
            instruction,
            *intrinsic,
            arguments,
            result,
            all_values,
            available,
        ),
        Operation::MakeClosure {
            function: target,
            captures,
        } => validate_closure(
            program,
            function,
            block,
            instruction,
            *target,
            captures,
            result,
            all_values,
            available,
        ),
        Operation::MakeCell(value) => validate_cell_make(
            program,
            function,
            block,
            instruction,
            *value,
            result,
            all_values,
            available,
        ),
        Operation::CellRead(cell) => validate_cell_read(
            program,
            function,
            block,
            instruction,
            *cell,
            result,
            all_values,
            available,
        ),
        Operation::CellWrite { cell, value } => validate_cell_write(
            program,
            function,
            block,
            instruction,
            *cell,
            *value,
            all_values,
            available,
        ),
        Operation::SpawnTask { callee, arguments } => validate_spawn(
            program,
            function,
            block,
            instruction,
            *callee,
            arguments,
            result,
            all_values,
            available,
        ),
        Operation::SpawnDetachedTask { callee, arguments } => validate_call_value(
            program,
            function,
            block,
            instruction,
            *callee,
            arguments,
            None,
            all_values,
            available,
        ),
        Operation::Yield => Ok(()),
    }
}

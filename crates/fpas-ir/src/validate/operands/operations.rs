/// Program, position, and value environment of the instruction being validated.
#[derive(Clone, Copy)]
struct OperandScope<'a> {
    program: &'a Program,
    function: &'a Function,
    block: BlockId,
    instruction: usize,
    /// Types of every value defined in the function.
    all_values: &'a BTreeMap<ValueId, TypeId>,
    /// Values defined before this instruction.
    available: &'a BTreeSet<ValueId>,
}

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
        let scope = OperandScope {
            program,
            function,
            block: block.id,
            instruction: instruction_index,
            all_values,
            available: &available,
        };
        validate_operation(scope, &instruction.operation, instruction.result)?;
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

fn validate_operation(
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
    match operation {
        Operation::Const(constant) => {
            validate_constant(scope, constant, result)
        }
Operation::StoreLocalIndex {
    local,
    index,
    value,
} => validate_store_local_index(scope, *local, *index, *value),
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
            require_assignable(
                program,
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
            validate_binary(scope, *operation, left_ty, right_ty, result)
        }
        Operation::Unary { operation, operand } => {
            let operand_ty =
                value_type(function, block, instruction, *operand, all_values, available)?;
            validate_unary(scope, *operation, operand_ty, result)
        }
        Operation::CallDirect {
            function: target,
            arguments,
        } => validate_direct_call(scope, *target, arguments, result),
        Operation::CallValue { callee, arguments } => validate_call_value(scope, *callee, arguments, result),
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
            require_assignable(
                program,
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
        } => validate_store_global_index_path(scope, *global, *root, indexes, *value),
        Operation::MakeArray(_)
        | Operation::ArrayPush { .. }
        | Operation::ArrayPop { .. }
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
        | Operation::LoadEnumField { .. } => validate_p5(scope, operation, result),
        Operation::MakeRecord { layout, fields } => validate_record_make(scope, *layout, fields, result),
        Operation::LoadField {
            record,
            layout,
            field,
        } => validate_field_load(scope, *record, *layout, *field, result),
        Operation::StoreField {
            record,
            layout,
            field,
            value,
        } => validate_field_store(scope, *record, *layout, *field, *value),
        Operation::MakeEnum {
            layout,
            variant,
            fields,
        } => validate_enum_make(scope, *layout, *variant, fields, result),
        Operation::TestVariant {
            value,
            layout,
            variant,
        } => validate_variant_test(scope, *value, *layout, *variant, result),
        Operation::Intrinsic {
            intrinsic,
            arguments,
        } => validate_intrinsic(scope, *intrinsic, arguments, result),
        Operation::MakeClosure {
            function: target,
            captures,
        } => validate_closure(scope, *target, captures, result),
        Operation::MakeCell(value) => validate_cell_make(scope, *value, result),
        Operation::CellRead(cell) => validate_cell_read(scope, *cell, result),
        Operation::CellWrite { cell, value } => validate_cell_write(scope, *cell, *value),
        Operation::SpawnTask { callee, arguments } => validate_spawn(scope, *callee, arguments, result),
        Operation::SpawnDetachedTask { callee, arguments } => validate_call_value(scope, *callee, arguments, None),
        Operation::Yield => Ok(()),
    }
}

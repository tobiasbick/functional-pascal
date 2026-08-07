//! Function partitions, frame metadata, branches, reachability, and feature flags.

use std::collections::VecDeque;

use crate::{FunctionId, FunctionInfo, InstructionAddress, Opcode, ReturnConvention, limits};

use super::instruction::validate_instruction;
use super::{ValidationError, ValidationErrorKind};

pub(super) fn validate_functions(executable: &crate::Executable) -> Result<(), ValidationError> {
    if executable.entry != FunctionId::new(0) || executable.functions.is_empty() {
        return Err(ValidationError::executable(
            ValidationErrorKind::EntryFunction {
                actual: executable.entry.get(),
                functions: executable.functions.len(),
            },
        ));
    }
    let root = &executable.functions[0];
    if root.arity != 0
        || root.capture_count != 0
        || root.return_convention != ReturnConvention::Unit
    {
        return Err(ValidationError::function(
            executable,
            FunctionId::new(0),
            ValidationErrorKind::EntrySignature {
                arity: root.arity,
                captures: root.capture_count,
                return_convention: root.return_convention,
            },
        ));
    }

    let mut expected_start = 0_u32;
    for (index, function) in executable.functions.iter().enumerate() {
        let function_id = FunctionId::try_from_index(index).map_err(|_| {
            ValidationError::executable(ValidationErrorKind::ResourceLimit {
                resource: "functions",
                actual: executable.functions.len(),
                maximum: limits::MAX_FUNCTIONS,
            })
        })?;
        validate_function_range(executable, function_id, function, expected_start)?;
        validate_frame(executable, function_id, function)?;
        validate_function_code(executable, function_id, function)?;
        expected_start = function.code.end.get();
    }
    let code_end = u32::try_from(executable.code.len()).map_err(|_| {
        ValidationError::executable(ValidationErrorKind::ResourceLimit {
            resource: "instructions",
            actual: executable.code.len(),
            maximum: limits::MAX_INSTRUCTIONS,
        })
    })?;
    if expected_start != code_end {
        return Err(ValidationError::executable(
            ValidationErrorKind::FunctionPartition {
                expected_start: code_end,
                actual_start: expected_start,
            },
        ));
    }
    Ok(())
}

fn validate_function_range(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
    expected_start: u32,
) -> Result<(), ValidationError> {
    if function.code.is_empty() {
        return Err(ValidationError::function(
            executable,
            function_id,
            ValidationErrorKind::EmptyCodeRange {
                start: function.code.start.get(),
                end: function.code.end.get(),
            },
        ));
    }
    let end = usize::try_from(function.code.end.get()).ok();
    if end.is_none_or(|end| end > executable.code.len()) {
        return Err(ValidationError::function(
            executable,
            function_id,
            ValidationErrorKind::CodeRange {
                start: function.code.start.get(),
                end: function.code.end.get(),
                code: executable.code.len(),
            },
        ));
    }
    if function.code.start.get() != expected_start {
        return Err(ValidationError::function(
            executable,
            function_id,
            ValidationErrorKind::FunctionPartition {
                expected_start,
                actual_start: function.code.start.get(),
            },
        ));
    }
    let length = function
        .code
        .len()
        .and_then(|length| usize::try_from(length).ok());
    if length.is_none_or(|length| length > limits::MAX_FUNCTION_INSTRUCTIONS) {
        return Err(ValidationError::function(
            executable,
            function_id,
            ValidationErrorKind::ResourceLimit {
                resource: "function instructions",
                actual: length.unwrap_or(usize::MAX),
                maximum: limits::MAX_FUNCTION_INSTRUCTIONS,
            },
        ));
    }
    Ok(())
}

fn validate_frame(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
) -> Result<(), ValidationError> {
    let required = usize::from(function.arity).checked_add(usize::from(function.capture_count));
    if usize::from(function.register_count) > limits::MAX_REGISTERS_PER_FUNCTION
        || usize::from(function.capture_count) > limits::MAX_CLOSURE_CAPTURES
        || required.is_none_or(|required| required > usize::from(function.register_count))
    {
        return Err(ValidationError::function(
            executable,
            function_id,
            ValidationErrorKind::FrameWindow {
                arity: function.arity,
                captures: function.capture_count,
                registers: function.register_count,
            },
        ));
    }
    Ok(())
}

fn validate_function_code(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
) -> Result<(), ValidationError> {
    let mut emitted_spawn = false;
    let mut raw_address = function.code.start.get();
    while raw_address < function.code.end.get() {
        let address = InstructionAddress::new(raw_address);
        let opcode = validate_instruction(executable, function_id, function, address)?;
        emitted_spawn |= matches!(opcode, Opcode::SpawnTask | Opcode::SpawnDetachedTask);
        raw_address = raw_address.checked_add(1).ok_or_else(|| {
            ValidationError::function(
                executable,
                function_id,
                ValidationErrorKind::CodeRange {
                    start: function.code.start.get(),
                    end: function.code.end.get(),
                    code: executable.code.len(),
                },
            )
        })?;
    }
    if emitted_spawn != function.flags.uses_spawn_tasks {
        return Err(ValidationError::function(
            executable,
            function_id,
            ValidationErrorKind::SpawnFlag {
                declared: function.flags.uses_spawn_tasks,
                emitted: emitted_spawn,
            },
        ));
    }
    validate_reachable_control_flow(executable, function_id, function)
}

fn validate_reachable_control_flow(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
) -> Result<(), ValidationError> {
    let length = usize::try_from(function.code.len().unwrap_or(0)).unwrap_or(0);
    let mut reached = vec![false; length];
    let mut pending = VecDeque::from([function.code.start]);

    while let Some(address) = pending.pop_front() {
        let local = address.get().checked_sub(function.code.start.get());
        let Some(local) = local.and_then(|value| usize::try_from(value).ok()) else {
            return Err(branch_error(
                executable,
                function_id,
                function,
                address,
                address.get(),
            ));
        };
        let Some(seen) = reached.get_mut(local) else {
            return Err(branch_error(
                executable,
                function_id,
                function,
                address,
                address.get(),
            ));
        };
        if *seen {
            continue;
        }
        *seen = true;

        let Some(instruction) = usize::try_from(address.get())
            .ok()
            .and_then(|index| executable.code.get(index))
            .copied()
        else {
            return Err(branch_error(
                executable,
                function_id,
                function,
                address,
                address.get(),
            ));
        };
        let opcode = instruction.opcode().map_err(|error| {
            ValidationError::instruction(
                executable,
                function_id,
                address,
                None,
                ValidationErrorKind::Instruction(error),
            )
        })?;
        match opcode {
            Opcode::Jump => {
                let target = instruction
                    .abx_operands()
                    .map_err(|error| {
                        ValidationError::instruction(
                            executable,
                            function_id,
                            address,
                            Some(opcode),
                            ValidationErrorKind::Instruction(error),
                        )
                    })?
                    .bx;
                push_target(
                    executable,
                    function_id,
                    function,
                    address,
                    target,
                    &mut pending,
                )?;
            }
            Opcode::BranchIfFalse | Opcode::BranchIfTrue => {
                let target = instruction
                    .abx_operands()
                    .map_err(|error| {
                        ValidationError::instruction(
                            executable,
                            function_id,
                            address,
                            Some(opcode),
                            ValidationErrorKind::Instruction(error),
                        )
                    })?
                    .bx;
                push_target(
                    executable,
                    function_id,
                    function,
                    address,
                    target,
                    &mut pending,
                )?;
                push_fallthrough(executable, function_id, function, address, &mut pending)?;
            }
            Opcode::Return | Opcode::Panic => {}
            _ => push_fallthrough(executable, function_id, function, address, &mut pending)?,
        }
    }
    Ok(())
}

fn push_target(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
    address: InstructionAddress,
    target: u32,
    pending: &mut VecDeque<InstructionAddress>,
) -> Result<(), ValidationError> {
    let target_address = InstructionAddress::new(target);
    if !function.code.contains(target_address) {
        return Err(branch_error(
            executable,
            function_id,
            function,
            address,
            target,
        ));
    }
    pending.push_back(target_address);
    Ok(())
}

fn push_fallthrough(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
    address: InstructionAddress,
    pending: &mut VecDeque<InstructionAddress>,
) -> Result<(), ValidationError> {
    let next = address.get().checked_add(1);
    if next.is_none_or(|next| next >= function.code.end.get()) {
        return Err(ValidationError::instruction(
            executable,
            function_id,
            address,
            executable
                .code
                .get(usize::try_from(address.get()).unwrap_or(usize::MAX))
                .and_then(|instruction| instruction.opcode().ok()),
            ValidationErrorKind::Fallthrough,
        ));
    }
    if let Some(next) = next {
        pending.push_back(InstructionAddress::new(next));
    }
    Ok(())
}

fn branch_error(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
    address: InstructionAddress,
    target: u32,
) -> ValidationError {
    ValidationError::instruction(
        executable,
        function_id,
        address,
        executable
            .code
            .get(usize::try_from(address.get()).unwrap_or(usize::MAX))
            .and_then(|instruction| instruction.opcode().ok()),
        ValidationErrorKind::BranchTarget {
            target,
            start: function.code.start.get(),
            end: function.code.end.get(),
        },
    )
}

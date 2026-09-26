//! Total active-subset IR instruction selection into checked packed instructions.

mod aggregates;
mod consuming_moves;
mod intrinsics;
mod local_moves;
mod operators;

use self::operators::unary_opcode;

use std::collections::BTreeMap;

use fpas_bytecode::{Instruction, Opcode};
use fpas_ir::{
    BinaryOperation, Function, IrType, Operation, Program, TypeId, UnaryOperation, ValueId,
};

use crate::CompileError;
use crate::error::internal_compiler_error;

use super::allocation::Allocation;
use super::metadata::MetadataBuilder;
use super::superinstructions::integer_immediate;

pub(super) struct Selector<'a> {
    program: &'a Program,
    allocation: &'a Allocation,
    value_types: BTreeMap<ValueId, TypeId>,
}

impl<'a> Selector<'a> {
    pub fn new(program: &'a Program, function: &Function, allocation: &'a Allocation) -> Self {
        let value_types = function
            .parameters
            .iter()
            .copied()
            .chain(
                function
                    .blocks
                    .iter()
                    .flat_map(|block| block.instructions.iter())
                    .filter_map(|instruction| instruction.result),
            )
            .map(|result| (result.id, result.ty))
            .collect();
        Self {
            program,
            allocation,
            value_types,
        }
    }

    /// Select the register instructions for instruction `index` of `block`.
    pub fn select(
        &self,
        block: &fpas_ir::BasicBlock,
        index: usize,
        metadata: &mut MetadataBuilder,
    ) -> Result<Vec<Instruction>, CompileError> {
        let instruction = block
            .instructions
            .get(index)
            .ok_or_else(|| selection_error("instruction index is outside its block"))?;
        let words = self.select_words(block, index, instruction, metadata)?;
        self.mark_consuming_moves(words, block.id, index, instruction)
    }

    fn select_words(
        &self,
        block: &fpas_ir::BasicBlock,
        index: usize,
        instruction: &fpas_ir::Instruction,
        metadata: &mut MetadataBuilder,
    ) -> Result<Vec<Instruction>, CompileError> {
        let previous = index
            .checked_sub(1)
            .and_then(|index| block.instructions.get(index));
        let result = instruction.result.map(|value| value.id);
        if let Some(selected) = self.select_aggregate(&instruction.operation, result, metadata)? {
            return Ok(selected);
        }
        if let Some(selected) = self.select_intrinsic(&instruction.operation, result)? {
            return Ok(selected);
        }
        let selected = match &instruction.operation {
            Operation::Const(_)
                if result.is_some_and(|result| self.allocation.is_dead_constant(result)) =>
            {
                return Ok(Vec::new());
            }
            Operation::Const(constant) => {
                let destination = self.result_register(result)?;
                if let Some(constant) = metadata.constant(constant)? {
                    abx(Opcode::LoadConstant, destination, constant.get())
                } else {
                    abc(Opcode::LoadUnit, destination, 0, 0)
                }
            }
            Operation::ReadLocal(local) => return self.select_read_local(*local, result),
            Operation::WriteLocal { value, local } => {
                return self.select_write_local(*value, *local);
            }
            Operation::Unary { operation, operand } => abc(
                unary_opcode(*operation),
                self.result_register(result)?,
                self.allocation.value(*operand)?.get(),
                0,
            ),
            Operation::Binary {
                operation,
                left,
                right,
            } => {
                let immediate = integer_immediate(previous, *operation, *left, *right);
                let opcode = match (operation, immediate) {
                    (BinaryOperation::AddInteger, Some(_)) => Opcode::AddIntegerImm,
                    (BinaryOperation::DivideInteger, Some(_)) => Opcode::DivideIntegerImm,
                    _ => self.binary_opcode(*operation, *left)?,
                };
                // Auxiliary 1 lets ConcatString reuse a left temporary that dies here.
                let consumes_left = opcode == Opcode::ConcatString
                    && left != right
                    && self.allocation.is_final_read(*left, block.id, index);
                abc_aux(
                    opcode,
                    self.result_register(result)?,
                    self.allocation.value(*left)?.get(),
                    match immediate {
                        Some(value) => value as u16,
                        None => self.allocation.value(*right)?.get(),
                    },
                    u8::from(consumes_left),
                )
            }
            Operation::CallDirect {
                function,
                arguments,
            } => return self.select_direct_call(*function, arguments, result),
            Operation::CallValue { callee, arguments } => {
                return self.select_value_call(*callee, arguments, result);
            }
            Operation::MakeClosure { function, captures } => {
                return self.select_closure(*function, captures, result);
            }
            Operation::MakeCell(value) => abc(
                Opcode::MakeCell,
                self.result_register(result)?,
                self.allocation.value(*value)?.get(),
                0,
            ),
            Operation::CellRead(value) => abc(
                Opcode::CellRead,
                self.result_register(result)?,
                self.allocation.value(*value)?.get(),
                0,
            ),
            Operation::CellWrite { cell, value } => abc(
                Opcode::CellWrite,
                self.allocation.value(*cell)?.get(),
                self.allocation.value(*value)?.get(),
                0,
            ),
            Operation::SpawnTask { callee, arguments } => {
                return self.select_spawn(*callee, arguments, result, false);
            }
            Operation::SpawnDetachedTask { callee, arguments } => {
                return self.select_spawn(*callee, arguments, result, true);
            }
            Operation::Yield => abc(Opcode::Yield, 0, 0, 0),
            other => Err(selection_error(&format!(
                "IR operation {other:?} has no bytecode selection"
            ))),
        }?;
        Ok(vec![selected])
    }

    fn select_spawn(
        &self,
        callee: ValueId,
        arguments: &[ValueId],
        result: Option<ValueId>,
        detached: bool,
    ) -> Result<Vec<Instruction>, CompileError> {
        let mut instructions = self.prepare_window(arguments)?;
        let (opcode, a, b, c) = if detached {
            (
                Opcode::SpawnDetachedTask,
                self.allocation.value(callee)?.get(),
                self.allocation.call_window().get(),
                0,
            )
        } else {
            (
                Opcode::SpawnTask,
                self.result_register(result)?,
                self.allocation.value(callee)?.get(),
                self.allocation.call_window().get(),
            )
        };
        instructions.push(abc_aux(opcode, a, b, c, argument_count(arguments)?)?);
        Ok(instructions)
    }

    fn select_direct_call(
        &self,
        function: fpas_ir::FunctionId,
        arguments: &[ValueId],
        result: Option<ValueId>,
    ) -> Result<Vec<Instruction>, CompileError> {
        let target = self
            .program
            .function(function)
            .ok_or_else(|| selection_error("direct call target is missing"))?;
        let target_id = u16::try_from(function.get())
            .map_err(|_| selection_error("function identifier exceeds u16"))?;
        let (mut instructions, argument_base) = self.prepare_call_window(arguments)?;
        let returns_unit = matches!(
            self.program
                .ty(target.signature.result)
                .map(|definition| &definition.kind),
            Some(IrType::Unit)
        );
        let destination = if returns_unit {
            fpas_bytecode::NO_REGISTER
        } else {
            self.result_register(result)?
        };
        instructions.push(abc_aux(
            Opcode::CallDirect,
            destination,
            target_id,
            argument_base,
            argument_count(arguments)?,
        )?);
        if returns_unit {
            instructions.push(abc(Opcode::LoadUnit, self.result_register(result)?, 0, 0)?);
        }
        Ok(instructions)
    }

    fn select_value_call(
        &self,
        callee: ValueId,
        arguments: &[ValueId],
        result: Option<ValueId>,
    ) -> Result<Vec<Instruction>, CompileError> {
        let (mut instructions, argument_base) = self.prepare_call_window(arguments)?;
        instructions.push(abc_aux(
            Opcode::CallValue,
            self.result_register(result)?,
            self.allocation.value(callee)?.get(),
            argument_base,
            argument_count(arguments)?,
        )?);
        Ok(instructions)
    }

    fn select_closure(
        &self,
        function: fpas_ir::FunctionId,
        captures: &[ValueId],
        result: Option<ValueId>,
    ) -> Result<Vec<Instruction>, CompileError> {
        let target_id = u16::try_from(function.get())
            .map_err(|_| selection_error("function identifier exceeds u16"))?;
        let mut instructions = self.prepare_window(captures)?;
        instructions.push(abc_aux(
            Opcode::MakeClosure,
            self.result_register(result)?,
            target_id,
            self.allocation.call_window().get(),
            argument_count(captures)?,
        )?);
        Ok(instructions)
    }

    pub(super) fn prepare_window(
        &self,
        values: &[ValueId],
    ) -> Result<Vec<Instruction>, CompileError> {
        self.prepare_window_at(self.allocation.call_window().get(), values)
    }

    fn prepare_window_at(
        &self,
        base: u16,
        values: &[ValueId],
    ) -> Result<Vec<Instruction>, CompileError> {
        values
            .iter()
            .enumerate()
            .map(|(offset, value)| {
                let destination = base
                    .checked_add(
                        u16::try_from(offset)
                            .map_err(|_| selection_error("call window offset exceeds u16"))?,
                    )
                    .ok_or_else(|| selection_error("call window exceeds u16"))?;
                abc(
                    Opcode::Move,
                    destination,
                    self.allocation.value(*value)?.get(),
                    0,
                )
            })
            .collect()
    }

    fn prepare_call_window(
        &self,
        values: &[ValueId],
    ) -> Result<(Vec<Instruction>, u16), CompileError> {
        if let [value] = values {
            return Ok((Vec::new(), self.allocation.value(*value)?.get()));
        }
        // Arguments end at the frame's last register, so the VM can start the callee frame on
        // them instead of copying (overlapping register windows).
        let base = self
            .allocation
            .register_count
            .checked_sub(
                u16::try_from(values.len())
                    .map_err(|_| selection_error("call argument count exceeds u16"))?,
            )
            .filter(|base| *base >= self.allocation.call_window().get())
            .ok_or_else(|| selection_error("call arguments exceed the call window"))?;
        Ok((self.prepare_window_at(base, values)?, base))
    }

    pub(super) fn result_register(&self, result: Option<ValueId>) -> Result<u16, CompileError> {
        result
            .ok_or_else(|| selection_error("value-producing IR operation has no result"))
            .and_then(|value| self.allocation.value(value).map(|register| register.get()))
    }
}

fn narrow(value: impl TryInto<u16>, kind: &str) -> Result<u16, CompileError> {
    value
        .try_into()
        .map_err(|_| selection_error(&format!("{kind} identifier exceeds u16")))
}

pub(super) fn abc(opcode: Opcode, a: u16, b: u16, c: u16) -> Result<Instruction, CompileError> {
    Instruction::abc(opcode, a, b, c, 0).map_err(|error| selection_error(&error.to_string()))
}

pub(super) fn abc_aux(
    opcode: Opcode,
    a: u16,
    b: u16,
    c: u16,
    auxiliary: u8,
) -> Result<Instruction, CompileError> {
    Instruction::abc(opcode, a, b, c, auxiliary)
        .map_err(|error| selection_error(&error.to_string()))
}

pub(super) fn argument_count(values: &[ValueId]) -> Result<u8, CompileError> {
    u8::try_from(values.len()).map_err(|_| selection_error("call or capture count exceeds u8"))
}

pub(super) fn abx(opcode: Opcode, a: u16, bx: u32) -> Result<Instruction, CompileError> {
    Instruction::abx(opcode, a, bx).map_err(|error| selection_error(&error.to_string()))
}

pub(super) fn selection_error(message: &str) -> CompileError {
    internal_compiler_error(
        format!("Register instruction selection failed: {message}."),
        "This is an internal compiler error. Re-run compilation and report the source program.",
        1,
        1,
    )
}

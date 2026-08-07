//! Total active-subset IR instruction selection into checked packed instructions.

use std::collections::BTreeMap;

use fpas_bytecode::{Instruction, Opcode};
use fpas_ir::{
    BinaryOperation, Function, IrType, Operation, Program, TypeId, UnaryOperation, ValueId,
};

use crate::CompileError;
use crate::error::internal_compiler_error;

use super::allocation::Allocation;
use super::metadata::MetadataBuilder;

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

    pub fn select(
        &self,
        instruction: &fpas_ir::Instruction,
        metadata: &mut MetadataBuilder,
    ) -> Result<Vec<Instruction>, CompileError> {
        let result = instruction.result.map(|value| value.id);
        let selected = match &instruction.operation {
            Operation::Const(constant) => {
                let destination = self.result_register(result)?;
                if let Some(constant) = metadata.constant(constant)? {
                    abx(Opcode::LoadConstant, destination, constant.get())
                } else {
                    abc(Opcode::LoadUnit, destination, 0, 0)
                }
            }
            Operation::ReadLocal(local) => abc(
                Opcode::Move,
                self.result_register(result)?,
                self.allocation.local(*local)?.get(),
                0,
            ),
            Operation::WriteLocal { value, local } => abc(
                Opcode::Move,
                self.allocation.local(*local)?.get(),
                self.allocation.value(*value)?.get(),
                0,
            ),
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
            } => abc(
                self.binary_opcode(*operation, *left)?,
                self.result_register(result)?,
                self.allocation.value(*left)?.get(),
                self.allocation.value(*right)?.get(),
            ),
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
            other => Err(selection_error(&format!(
                "IR operation {other:?} belongs to a later register-VM phase"
            ))),
        }?;
        Ok(vec![selected])
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
        let mut instructions = self.prepare_window(arguments)?;
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
            self.allocation.call_window().get(),
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
        let mut instructions = self.prepare_window(arguments)?;
        instructions.push(abc_aux(
            Opcode::CallValue,
            self.result_register(result)?,
            self.allocation.value(callee)?.get(),
            self.allocation.call_window().get(),
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

    fn prepare_window(&self, values: &[ValueId]) -> Result<Vec<Instruction>, CompileError> {
        let base = self.allocation.call_window().get();
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

    fn binary_opcode(
        &self,
        operation: BinaryOperation,
        left: ValueId,
    ) -> Result<Opcode, CompileError> {
        let direct = match operation {
            BinaryOperation::AddInteger => Some(Opcode::AddInteger),
            BinaryOperation::SubtractInteger => Some(Opcode::SubtractInteger),
            BinaryOperation::MultiplyInteger => Some(Opcode::MultiplyInteger),
            BinaryOperation::DivideInteger => Some(Opcode::DivideInteger),
            BinaryOperation::RemainderInteger => Some(Opcode::RemainderInteger),
            BinaryOperation::AddReal => Some(Opcode::AddReal),
            BinaryOperation::SubtractReal => Some(Opcode::SubtractReal),
            BinaryOperation::MultiplyReal => Some(Opcode::MultiplyReal),
            BinaryOperation::DivideReal => Some(Opcode::DivideReal),
            BinaryOperation::AddDynamic => Some(Opcode::AddDynamic),
            BinaryOperation::SubtractDynamic => Some(Opcode::SubtractDynamic),
            BinaryOperation::MultiplyDynamic => Some(Opcode::MultiplyDynamic),
            BinaryOperation::DivideDynamic => Some(Opcode::DivideDynamic),
            BinaryOperation::LessThanInteger => Some(Opcode::LessInteger),
            BinaryOperation::GreaterThanInteger => Some(Opcode::GreaterInteger),
            BinaryOperation::LessEqualInteger => Some(Opcode::LessEqualInteger),
            BinaryOperation::GreaterEqualInteger => Some(Opcode::GreaterEqualInteger),
            BinaryOperation::LessThanReal => Some(Opcode::LessReal),
            BinaryOperation::GreaterThanReal => Some(Opcode::GreaterReal),
            BinaryOperation::LessEqualReal => Some(Opcode::LessEqualReal),
            BinaryOperation::GreaterEqualReal => Some(Opcode::GreaterEqualReal),
            BinaryOperation::LessThanDynamic => Some(Opcode::LessDynamic),
            BinaryOperation::GreaterThanDynamic => Some(Opcode::GreaterDynamic),
            BinaryOperation::LessEqualDynamic => Some(Opcode::LessEqualDynamic),
            BinaryOperation::GreaterEqualDynamic => Some(Opcode::GreaterEqualDynamic),
            BinaryOperation::AndBoolean => Some(Opcode::AndBoolean),
            BinaryOperation::OrBoolean => Some(Opcode::OrBoolean),
            BinaryOperation::ConcatString => Some(Opcode::ConcatString),
            BinaryOperation::ShiftLeftInteger => Some(Opcode::ShiftLeftInteger),
            BinaryOperation::ShiftRightInteger => Some(Opcode::ShiftRightInteger),
            BinaryOperation::BitAndInteger => Some(Opcode::BitAndInteger),
            BinaryOperation::BitOrInteger => Some(Opcode::BitOrInteger),
            BinaryOperation::BitXorInteger => Some(Opcode::BitXorInteger),
            BinaryOperation::Equal | BinaryOperation::NotEqual => None,
        };
        if let Some(opcode) = direct {
            return Ok(opcode);
        }
        let ty = self
            .value_types
            .get(&left)
            .and_then(|ty| self.program.ty(*ty))
            .map(|definition| &definition.kind)
            .ok_or_else(|| selection_error("equality operand type is missing"))?;
        let equal = operation == BinaryOperation::Equal;
        match (ty, equal) {
            (IrType::Integer, true) => Ok(Opcode::EqualInteger),
            (IrType::Integer, false) => Ok(Opcode::NotEqualInteger),
            (IrType::Real, true) => Ok(Opcode::EqualReal),
            (IrType::Real, false) => Ok(Opcode::NotEqualReal),
            (IrType::Boolean, true) => Ok(Opcode::EqualBoolean),
            (IrType::Boolean, false) => Ok(Opcode::NotEqualBoolean),
            (IrType::String, true) => Ok(Opcode::EqualString),
            (IrType::String, false) => Ok(Opcode::NotEqualString),
            (IrType::Dynamic, true) => Ok(Opcode::EqualDynamic),
            (IrType::Dynamic, false) => Ok(Opcode::NotEqualDynamic),
            _ => Err(selection_error("equality operand is not a P3 scalar type")),
        }
    }

    fn result_register(&self, result: Option<ValueId>) -> Result<u16, CompileError> {
        result
            .ok_or_else(|| selection_error("value-producing IR operation has no result"))
            .and_then(|value| self.allocation.value(value).map(|register| register.get()))
    }
}

fn unary_opcode(operation: UnaryOperation) -> Opcode {
    match operation {
        UnaryOperation::NegateInteger => Opcode::NegateInteger,
        UnaryOperation::NegateReal => Opcode::NegateReal,
        UnaryOperation::NegateDynamic => Opcode::NegateDynamic,
        UnaryOperation::NotBoolean => Opcode::NotBoolean,
        UnaryOperation::IntegerToReal => Opcode::IntegerToReal,
    }
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

fn argument_count(values: &[ValueId]) -> Result<u8, CompileError> {
    u8::try_from(values.len()).map_err(|_| selection_error("call or capture count exceeds u8"))
}

pub(super) fn abx(opcode: Opcode, a: u16, bx: u32) -> Result<Instruction, CompileError> {
    Instruction::abx(opcode, a, bx).map_err(|error| selection_error(&error.to_string()))
}

fn selection_error(message: &str) -> CompileError {
    internal_compiler_error(
        format!("Register instruction selection failed: {message}."),
        "This is an internal compiler error. Re-run compilation and report the source program.",
        1,
        1,
    )
}

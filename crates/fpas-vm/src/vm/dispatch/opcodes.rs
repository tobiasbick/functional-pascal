//! Exhaustive opcode handlers for one pre-decoded instruction.

use fpas_bytecode::{DecodedInstruction, NO_REGISTER, Opcode, Value};
use fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC;

use super::super::VmError;
use super::super::value_ops::{self, BinaryOperation, UnaryOperation};
use super::super::worker::Worker;
use super::{DispatchStep, Flow};

impl Worker {
    /// Execute one verified instruction whose address is already recorded.
    #[inline(always)]
    pub(super) fn execute(&mut self, instruction: DecodedInstruction) -> Result<Flow, VmError> {
        match instruction.opcode() {
            Opcode::LoadConstant => {
                let operands = instruction.abx();
                let value = self.load_constant(operands.bx)?;
                self.write_operand(operands.a, value)?;
            }
            Opcode::LoadUnit => self.write_operand(instruction.abc().a, Value::Unit)?,
            Opcode::Move => {
                let operands = instruction.abc();
                // Auxiliary 1: the compiler proved the source dead, so transfer instead of clone.
                let value = if operands.auxiliary == 1 {
                    self.take_register(self.base + usize::from(operands.b))?
                } else {
                    self.read_operand(operands.b)?.clone()
                };
                self.write_operand(operands.a, value)?;
            }
            Opcode::AddInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::Add)?
            }
            Opcode::AddIntegerImm => {
                self.execute_integer_immediate(instruction.abc(), BinaryOperation::Add)?
            }
            Opcode::AddReal => self.execute_real_binary(instruction.abc(), BinaryOperation::Add)?,
            Opcode::AddDynamic => {
                self.execute_value_binary(instruction.abc(), BinaryOperation::Add)?
            }
            Opcode::ConcatString => self.execute_string_concat(instruction.abc())?,
            Opcode::SubtractInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::Subtract)?
            }
            Opcode::SubtractReal => {
                self.execute_real_binary(instruction.abc(), BinaryOperation::Subtract)?
            }
            Opcode::SubtractDynamic => {
                self.execute_value_binary(instruction.abc(), BinaryOperation::Subtract)?
            }
            Opcode::MultiplyInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::Multiply)?
            }
            Opcode::MultiplyReal => {
                self.execute_real_binary(instruction.abc(), BinaryOperation::Multiply)?
            }
            Opcode::MultiplyDynamic => {
                self.execute_value_binary(instruction.abc(), BinaryOperation::Multiply)?
            }
            Opcode::DivideInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::IntegerDivide)?
            }
            Opcode::DivideIntegerImm => {
                self.execute_integer_immediate(instruction.abc(), BinaryOperation::IntegerDivide)?
            }
            Opcode::RemainderInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::Modulo)?
            }
            Opcode::DivideReal => {
                self.execute_real_binary(instruction.abc(), BinaryOperation::RealDivide)?
            }
            Opcode::DivideDynamic => {
                self.execute_value_binary(instruction.abc(), BinaryOperation::RealDivide)?
            }
            Opcode::NegateInteger => {
                self.execute_integer_unary(instruction.abc(), UnaryOperation::Negate)?
            }
            Opcode::NegateReal | Opcode::NegateDynamic => {
                self.execute_value_unary(instruction.abc(), UnaryOperation::Negate)?
            }
            Opcode::EqualDynamic => {
                self.execute_value_binary(instruction.abc(), BinaryOperation::Equal)?
            }
            Opcode::NotEqualDynamic => {
                self.execute_value_binary(instruction.abc(), BinaryOperation::NotEqual)?
            }
            Opcode::LessDynamic => {
                self.execute_value_binary(instruction.abc(), BinaryOperation::Less)?
            }
            Opcode::GreaterDynamic => {
                self.execute_value_binary(instruction.abc(), BinaryOperation::Greater)?
            }
            Opcode::LessEqualDynamic => {
                self.execute_value_binary(instruction.abc(), BinaryOperation::LessEqual)?
            }
            Opcode::GreaterEqualDynamic => {
                self.execute_value_binary(instruction.abc(), BinaryOperation::GreaterEqual)?
            }
            Opcode::ShiftLeftInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::ShiftLeft)?
            }
            Opcode::ShiftRightInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::ShiftRight)?
            }
            Opcode::BitAndInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::And)?
            }
            Opcode::BitOrInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::Or)?
            }
            Opcode::BitXorInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::Xor)?
            }
            Opcode::EqualInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::Equal)?
            }
            Opcode::EqualReal => {
                self.execute_real_comparison(instruction.abc(), BinaryOperation::Equal)?
            }
            Opcode::EqualString => {
                self.execute_string_comparison(instruction.abc(), BinaryOperation::Equal)?
            }
            Opcode::EqualBoolean => {
                self.execute_boolean_binary(instruction.abc(), BinaryOperation::Equal)?
            }
            Opcode::NotEqualInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::NotEqual)?
            }
            Opcode::NotEqualReal => {
                self.execute_real_comparison(instruction.abc(), BinaryOperation::NotEqual)?
            }
            Opcode::NotEqualString => {
                self.execute_string_comparison(instruction.abc(), BinaryOperation::NotEqual)?
            }
            Opcode::NotEqualBoolean => {
                self.execute_boolean_binary(instruction.abc(), BinaryOperation::NotEqual)?
            }
            Opcode::LessInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::Less)?
            }
            Opcode::LessReal => {
                self.execute_real_comparison(instruction.abc(), BinaryOperation::Less)?
            }
            Opcode::LessString => {
                self.execute_string_comparison(instruction.abc(), BinaryOperation::Less)?
            }
            Opcode::GreaterInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::Greater)?
            }
            Opcode::GreaterReal => {
                self.execute_real_comparison(instruction.abc(), BinaryOperation::Greater)?
            }
            Opcode::GreaterString => {
                self.execute_string_comparison(instruction.abc(), BinaryOperation::Greater)?
            }
            Opcode::LessEqualInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::LessEqual)?
            }
            Opcode::LessEqualReal => {
                self.execute_real_comparison(instruction.abc(), BinaryOperation::LessEqual)?
            }
            Opcode::LessEqualString => {
                self.execute_string_comparison(instruction.abc(), BinaryOperation::LessEqual)?
            }
            Opcode::GreaterEqualInteger => {
                self.execute_integer_binary(instruction.abc(), BinaryOperation::GreaterEqual)?
            }
            Opcode::GreaterEqualReal => {
                self.execute_real_comparison(instruction.abc(), BinaryOperation::GreaterEqual)?
            }
            Opcode::GreaterEqualString => {
                self.execute_string_comparison(instruction.abc(), BinaryOperation::GreaterEqual)?
            }
            Opcode::NotBoolean => {
                self.execute_value_unary(instruction.abc(), UnaryOperation::Not)?
            }
            Opcode::AndBoolean => {
                self.execute_boolean_binary(instruction.abc(), BinaryOperation::And)?
            }
            Opcode::OrBoolean => {
                self.execute_boolean_binary(instruction.abc(), BinaryOperation::Or)?
            }
            Opcode::IntegerToReal => self.execute_integer_to_real(instruction.abc())?,
            Opcode::BranchIfEqualInteger
            | Opcode::BranchIfNotEqualInteger
            | Opcode::BranchIfLessInteger
            | Opcode::BranchIfGreaterInteger
            | Opcode::BranchIfLessEqualInteger
            | Opcode::BranchIfGreaterEqualInteger => {
                self.execute_integer_compare_branch(instruction)?;
            }
            Opcode::ForLoop => self.execute_for_loop(instruction.abc())?,
            Opcode::Jump => self.jump(instruction.abx().bx),
            Opcode::BranchIfFalse => {
                let operands = instruction.abx();
                if !self.branch_condition(operands.a)? {
                    self.jump(operands.bx);
                }
            }
            Opcode::BranchIfTrue => {
                let operands = instruction.abx();
                if self.branch_condition(operands.a)? {
                    self.jump(operands.bx);
                }
            }
            Opcode::Return => {
                let operands = instruction.abc();
                let value = if operands.a == NO_REGISTER {
                    Value::Unit
                } else {
                    self.read_operand(operands.a)?.clone()
                };
                return Ok(match self.return_from_call(value)? {
                    DispatchStep::Return(value) => Flow::Return(value),
                    DispatchStep::Continue | DispatchStep::Suspend => Flow::Boundary,
                });
            }
            Opcode::Panic => {
                let value = self.read_operand(instruction.abc().a)?.to_string();
                return Err(self.runtime_error(
                    RUNTIME_PROGRAM_PANIC,
                    format!("panic: {value}"),
                    "Remove the panic or guard the failing condition before calling panic.",
                ));
            }
            Opcode::CallDirect => self.call_direct(instruction.abc())?,
            // Debugger-owned workers keep every frame; the following Return word then returns
            // the callee's result as an ordinary call would.
            Opcode::TailCall if self.debug_tasks => self.call_direct(instruction.abc())?,
            Opcode::TailCall => self.tail_call(instruction.abc())?,
            Opcode::CallValue => self.call_value(instruction.abc())?,
            Opcode::MakeClosure => self.make_closure(instruction.abc())?,
            Opcode::MakeCell => self.make_cell(instruction.abc())?,
            Opcode::CellRead => self.read_cell(instruction.abc())?,
            Opcode::CellWrite => self.write_cell(instruction.abc())?,
            Opcode::LoadGlobal => self.load_global(instruction.abx())?,
            Opcode::StoreGlobal => self.store_global(instruction.abx())?,
            Opcode::StoreGlobalIndexPath => self.store_global_index_path(instruction.abc())?,
            Opcode::MakeArray => self.make_array(instruction.abc())?,
            Opcode::ArrayPop => self.array_pop(instruction.abc())?,
            Opcode::ArrayPush => self.array_push(instruction.abc())?,
            Opcode::IndexGet => self.index_get(instruction.abc())?,
            Opcode::IndexSet => self.index_set(instruction.abc())?,
            Opcode::Contains => self.contains(instruction.abc())?,
            Opcode::MakeDictionary => self.make_dictionary(instruction.abc())?,
            Opcode::MakeRecord => self.make_record(instruction.abc())?,
            Opcode::LoadField => self.load_field(instruction.abc())?,
            Opcode::StoreField => self.store_field(instruction.abc())?,
            Opcode::UpdateRecord => self.update_record(instruction.abc())?,
            Opcode::MakeOk => self.wrap(instruction.abc(), Value::result_ok)?,
            Opcode::MakeError => self.wrap(instruction.abc(), Value::result_error)?,
            Opcode::MakeSome => self.wrap(instruction.abc(), Value::option_some)?,
            Opcode::MakeNone => self.none(instruction.abc())?,
            Opcode::IsResultOk => self.test_ok(instruction.abc())?,
            Opcode::IsOptionSome => self.test_some(instruction.abc())?,
            Opcode::UnwrapOk => self.unwrap(instruction.abc(), "Ok")?,
            Opcode::UnwrapError => self.unwrap(instruction.abc(), "Error")?,
            Opcode::UnwrapSome => self.unwrap(instruction.abc(), "Some")?,
            Opcode::MakeEnum => self.make_enum(instruction.abc())?,
            Opcode::TestVariant => self.test_variant(instruction.abc())?,
            Opcode::LoadEnumField => self.load_enum_field(instruction.abc())?,
            Opcode::Intrinsic => {
                self.execute_intrinsic(instruction.abc())?;
                return Ok(Flow::Boundary);
            }
            Opcode::SpawnTask => {
                self.spawn_task(instruction.abc(), false)?;
                return Ok(Flow::Boundary);
            }
            Opcode::SpawnDetachedTask => {
                self.spawn_task(instruction.abc(), true)?;
                return Ok(Flow::Boundary);
            }
            Opcode::Yield => {
                self.yield_task();
                return Ok(Flow::Boundary);
            }
        }
        Ok(Flow::Next)
    }

    /// Move to a verified branch target; the verifier keeps targets inside the current function.
    #[inline(always)]
    fn jump(&mut self, address: u32) {
        self.ip = address as usize;
    }

    #[inline(always)]
    fn branch_condition(&self, operand: u16) -> Result<bool, VmError> {
        Ok(match self.read_operand(operand)? {
            Value::Boolean(value) => *value,
            Value::Integer(value) => *value != 0,
            Value::Unit | Value::OptionNone => false,
            _ => true,
        })
    }

    #[inline(always)]
    fn execute_integer_immediate(
        &mut self,
        operands: fpas_bytecode::AbcOperands,
        operation: BinaryOperation,
    ) -> Result<(), VmError> {
        let left = self.read_operand(operands.b)?;
        let immediate = i64::from(operands.c as i16);
        let result = match left {
            Value::Integer(value) => value_ops::integer_binary(operation, *value, immediate),
            _ => value_ops::binary(operation, left, &Value::Integer(immediate)),
        }
        .map_err(|error| self.runtime_error(error.code, error.message, error.hint))?;
        self.write_operand(operands.a, result)
    }

    #[inline(always)]
    fn execute_integer_compare_branch(
        &mut self,
        instruction: DecodedInstruction,
    ) -> Result<(), VmError> {
        let operands = instruction.abc();
        let operation = match instruction.opcode() {
            Opcode::BranchIfEqualInteger => BinaryOperation::Equal,
            Opcode::BranchIfNotEqualInteger => BinaryOperation::NotEqual,
            Opcode::BranchIfLessInteger => BinaryOperation::Less,
            Opcode::BranchIfGreaterInteger => BinaryOperation::Greater,
            Opcode::BranchIfLessEqualInteger => BinaryOperation::LessEqual,
            Opcode::BranchIfGreaterEqualInteger => BinaryOperation::GreaterEqual,
            _ => unreachable!("integer compare-and-branch opcode"),
        };
        self.execute_integer_binary(operands, operation)?;
        let condition = self.branch_condition(operands.a)?;
        let branch = self.executable.decoded()[self.ip];
        let target = branch.abx().bx;
        self.ip += 1;
        if condition == (branch.opcode() == Opcode::BranchIfTrue) {
            self.jump(target);
        }
        Ok(())
    }

    #[inline(always)]
    fn execute_for_loop(&mut self, operands: fpas_bytecode::AbcOperands) -> Result<(), VmError> {
        let counter = self.read_operand(operands.a)?;
        let bound = self.read_operand(operands.b)?;
        let (Value::Integer(counter), Value::Integer(bound)) = (counter, bound) else {
            return Err(self.runtime_error(
                fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                "Integer loop requires integer counter and bound",
                "Use integer values for the loop counter and bound.",
            ));
        };
        let (target, updated) = if counter == bound {
            (self.executable.decoded()[self.ip + 1].abx().bx, None)
        } else {
            (
                self.executable.decoded()[self.ip].abx().bx,
                Some(if operands.auxiliary == 0 {
                    counter.wrapping_add(1)
                } else {
                    counter.wrapping_sub(1)
                }),
            )
        };
        if let Some(updated) = updated {
            self.write_operand(operands.a, Value::Integer(updated))?;
        }
        self.jump(target);
        Ok(())
    }
}

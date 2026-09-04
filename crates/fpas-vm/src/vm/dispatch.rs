//! Single exhaustive packed-opcode dispatch loop.

use fpas_bytecode::{AbcOperands, AbxOperands, InstructionAddress, Opcode, Register, Value};
use fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC;

use super::execute::scalar::register;
use super::value_ops::{BinaryOperation, UnaryOperation};
use super::worker::Worker;
use super::{VmError, diagnostics};

pub(super) enum DispatchStep {
    Continue,
    Suspend,
    Return(Value),
}

impl Worker {
    /// Execute one instruction without debugger-only initializer suppression.
    pub fn dispatch_one(&mut self) -> Result<DispatchStep, VmError> {
        self.dispatch_one_with_initializer_suppression::<false>()
    }

    /// Execute one instruction for debugger-owned execution.
    pub(in crate::vm) fn dispatch_debug_one(&mut self) -> Result<DispatchStep, VmError> {
        self.dispatch_one_with_initializer_suppression::<true>()
    }

    fn dispatch_one_with_initializer_suppression<const SUPPRESS_INITIALIZERS: bool>(
        &mut self,
    ) -> Result<DispatchStep, VmError> {
        if !self.callback_continuations.is_empty() && self.resume_callback_continuation()? {
            return Ok(DispatchStep::Continue);
        }
        self.current_address = InstructionAddress::try_from_index(self.ip).map_err(|error| {
            diagnostics::internal(
                self.executable.executable(),
                InstructionAddress::new(0),
                format!("Instruction pointer is not portable: {error}"),
            )
        })?;
        let instruction = self
            .executable
            .executable()
            .code
            .get(self.ip)
            .copied()
            .ok_or_else(|| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Instruction pointer is outside verified code",
                )
            })?;
        self.ip = self.ip.checked_add(1).ok_or_else(|| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "Instruction pointer overflowed this host",
            )
        })?;
        self.instruction_count = self.instruction_count.checked_add(1).ok_or_else(|| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                "Instruction counter overflowed",
            )
        })?;
        if SUPPRESS_INITIALIZERS && self.take_suppressed_source_initializer(self.current_address) {
            return Ok(DispatchStep::Continue);
        }
        let opcode = instruction.opcode().map_err(|error| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Verified opcode failed decoding: {error}"),
            )
        })?;

        match opcode {
            Opcode::LoadConstant => {
                let operands = self.abx(instruction)?;
                let value = self.load_constant(operands.bx)?;
                self.write(register(operands.a)?, value)?;
            }
            Opcode::LoadUnit => {
                let operands = self.abc(instruction)?;
                self.write(register(operands.a)?, Value::Unit)?;
            }
            Opcode::Move => {
                let operands = self.abc(instruction)?;
                let value = self.read(register(operands.b)?)?.clone();
                self.write(register(operands.a)?, value)?;
            }
            Opcode::AddInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::Add)?
            }
            Opcode::AddReal | Opcode::AddDynamic | Opcode::ConcatString => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::Add)?
            }
            Opcode::SubtractInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::Subtract)?
            }
            Opcode::SubtractReal | Opcode::SubtractDynamic => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::Subtract)?
            }
            Opcode::MultiplyInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::Multiply)?
            }
            Opcode::MultiplyReal | Opcode::MultiplyDynamic => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::Multiply)?
            }
            Opcode::DivideInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::IntegerDivide)?
            }
            Opcode::RemainderInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::Modulo)?
            }
            Opcode::DivideReal | Opcode::DivideDynamic => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::RealDivide)?
            }
            Opcode::NegateInteger => {
                self.execute_integer_unary(self.abc(instruction)?, UnaryOperation::Negate)?
            }
            Opcode::NegateReal | Opcode::NegateDynamic => {
                self.execute_value_unary(self.abc(instruction)?, UnaryOperation::Negate)?
            }
            Opcode::EqualDynamic => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::Equal)?
            }
            Opcode::NotEqualDynamic => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::NotEqual)?
            }
            Opcode::LessDynamic => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::Less)?
            }
            Opcode::GreaterDynamic => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::Greater)?
            }
            Opcode::LessEqualDynamic => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::LessEqual)?
            }
            Opcode::GreaterEqualDynamic => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::GreaterEqual)?
            }
            Opcode::ShiftLeftInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::ShiftLeft)?
            }
            Opcode::ShiftRightInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::ShiftRight)?
            }
            Opcode::BitAndInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::And)?
            }
            Opcode::BitOrInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::Or)?
            }
            Opcode::BitXorInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::Xor)?
            }
            Opcode::EqualInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::Equal)?
            }
            Opcode::EqualReal | Opcode::EqualString | Opcode::EqualBoolean => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::Equal)?
            }
            Opcode::NotEqualInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::NotEqual)?
            }
            Opcode::NotEqualReal | Opcode::NotEqualString | Opcode::NotEqualBoolean => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::NotEqual)?
            }
            Opcode::LessInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::Less)?
            }
            Opcode::LessReal | Opcode::LessString => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::Less)?
            }
            Opcode::GreaterInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::Greater)?
            }
            Opcode::GreaterReal | Opcode::GreaterString => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::Greater)?
            }
            Opcode::LessEqualInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::LessEqual)?
            }
            Opcode::LessEqualReal | Opcode::LessEqualString => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::LessEqual)?
            }
            Opcode::GreaterEqualInteger => {
                self.execute_integer_binary(self.abc(instruction)?, BinaryOperation::GreaterEqual)?
            }
            Opcode::GreaterEqualReal | Opcode::GreaterEqualString => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::GreaterEqual)?
            }
            Opcode::NotBoolean => {
                self.execute_value_unary(self.abc(instruction)?, UnaryOperation::Not)?
            }
            Opcode::AndBoolean => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::And)?
            }
            Opcode::OrBoolean => {
                self.execute_value_binary(self.abc(instruction)?, BinaryOperation::Or)?
            }
            Opcode::IntegerToReal => self.execute_integer_to_real(self.abc(instruction)?)?,
            Opcode::Jump => {
                let operands = self.abx(instruction)?;
                self.set_ip(operands.bx)?;
            }
            Opcode::BranchIfFalse => {
                let operands = self.abx(instruction)?;
                if !self.branch_condition(operands.a)? {
                    self.set_ip(operands.bx)?;
                }
            }
            Opcode::BranchIfTrue => {
                let operands = self.abx(instruction)?;
                if self.branch_condition(operands.a)? {
                    self.set_ip(operands.bx)?;
                }
            }
            Opcode::Return => {
                let operands = self.abc(instruction)?;
                let value = if operands.a == fpas_bytecode::NO_REGISTER {
                    Value::Unit
                } else {
                    self.read(register(operands.a)?)?.clone()
                };
                return self.return_from_call(value);
            }
            Opcode::Panic => {
                let operands = self.abc(instruction)?;
                let value = self.read(register(operands.a)?)?.to_string();
                return Err(self.runtime_error(
                    RUNTIME_PROGRAM_PANIC,
                    format!("panic: {value}"),
                    "Remove the panic or guard the failing condition before calling panic.",
                ));
            }
            Opcode::CallDirect => self.call_direct(self.abc(instruction)?)?,
            Opcode::CallValue => self.call_value(self.abc(instruction)?)?,
            Opcode::MakeClosure => self.make_closure(self.abc(instruction)?)?,
            Opcode::MakeCell => self.make_cell(self.abc(instruction)?)?,
            Opcode::CellRead => self.read_cell(self.abc(instruction)?)?,
            Opcode::CellWrite => self.write_cell(self.abc(instruction)?)?,
            Opcode::LoadGlobal => self.load_global(self.abx(instruction)?)?,
            Opcode::StoreGlobal => self.store_global(self.abx(instruction)?)?,
            Opcode::StoreGlobalIndexPath => self.store_global_index_path(self.abc(instruction)?)?,
            Opcode::MakeArray => self.make_array(self.abc(instruction)?)?,
            Opcode::ArrayPush => self.array_push(self.abc(instruction)?)?,
            Opcode::IndexGet => self.index_get(self.abc(instruction)?)?,
            Opcode::IndexSet => self.index_set(self.abc(instruction)?)?,
            Opcode::Contains => self.contains(self.abc(instruction)?)?,
            Opcode::MakeDictionary => self.make_dictionary(self.abc(instruction)?)?,
            Opcode::MakeRecord => self.make_record(self.abc(instruction)?)?,
            Opcode::LoadField => self.load_field(self.abc(instruction)?)?,
            Opcode::StoreField => self.store_field(self.abc(instruction)?)?,
            Opcode::UpdateRecord => self.update_record(self.abc(instruction)?)?,
            Opcode::MakeOk => self.wrap(self.abc(instruction)?, Value::result_ok)?,
            Opcode::MakeError => self.wrap(self.abc(instruction)?, Value::result_error)?,
            Opcode::MakeSome => self.wrap(self.abc(instruction)?, Value::option_some)?,
            Opcode::MakeNone => self.none(self.abc(instruction)?)?,
            Opcode::IsResultOk => self.test_ok(self.abc(instruction)?)?,
            Opcode::IsOptionSome => self.test_some(self.abc(instruction)?)?,
            Opcode::UnwrapOk => self.unwrap(self.abc(instruction)?, "Ok")?,
            Opcode::UnwrapError => self.unwrap(self.abc(instruction)?, "Error")?,
            Opcode::UnwrapSome => self.unwrap(self.abc(instruction)?, "Some")?,
            Opcode::MakeEnum => self.make_enum(self.abc(instruction)?)?,
            Opcode::TestVariant => self.test_variant(self.abc(instruction)?)?,
            Opcode::LoadEnumField => self.load_enum_field(self.abc(instruction)?)?,
            Opcode::Intrinsic => self.execute_intrinsic(self.abc(instruction)?)?,
            Opcode::SpawnTask => self.spawn_task(self.abc(instruction)?, false)?,
            Opcode::SpawnDetachedTask => self.spawn_task(self.abc(instruction)?, true)?,
            Opcode::Yield => self.yield_task(),
        }
        if self.suspend_requested {
            return Ok(DispatchStep::Suspend);
        }
        Ok(DispatchStep::Continue)
    }

    fn abc(&self, instruction: fpas_bytecode::Instruction) -> Result<AbcOperands, VmError> {
        Ok(instruction.abc_payload())
    }

    fn abx(&self, instruction: fpas_bytecode::Instruction) -> Result<AbxOperands, VmError> {
        Ok(instruction.abx_payload())
    }

    fn set_ip(&mut self, address: u32) -> Result<(), VmError> {
        let target = InstructionAddress::new(address);
        let function = self
            .executable
            .executable()
            .functions
            .get(usize::from(self.function.get()))
            .ok_or_else(|| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Current function metadata is missing",
                )
            })?;
        if !function.code.contains(target) {
            return Err(diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Branch target {address} leaves the current function"),
            ));
        }
        self.ip = usize::try_from(address).map_err(|_| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Branch target {address} does not fit this host"),
            )
        })?;
        Ok(())
    }

    fn branch_condition(&self, register_value: u16) -> Result<bool, VmError> {
        match self.read(Register::new(register_value).map_err(|error| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Verified branch register failed decoding: {error}"),
            )
        })?)? {
            Value::Boolean(value) => Ok(*value),
            Value::Integer(value) => Ok(*value != 0),
            Value::Unit | Value::OptionNone => Ok(false),
            _ => Ok(true),
        }
    }
}

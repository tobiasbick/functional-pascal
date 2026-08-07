//! Single exhaustive packed-opcode dispatch loop.

use fpas_bytecode::{AbcOperands, AbxOperands, InstructionAddress, Opcode, Register, Value};
use fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC;

use super::execute::dynamic::DynamicArithmetic;
use super::execute::scalar::register;
use super::worker::RegisterWorker;
use super::{VmError, diagnostics};

pub(super) enum DispatchStep {
    Continue,
    Return(Value),
}

impl RegisterWorker {
    pub fn dispatch_one(&mut self) -> Result<DispatchStep, VmError> {
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
                self.execute_binary_integer(self.abc(instruction)?, i64::wrapping_add)?
            }
            Opcode::SubtractInteger => {
                self.execute_binary_integer(self.abc(instruction)?, i64::wrapping_sub)?
            }
            Opcode::MultiplyInteger => {
                self.execute_binary_integer(self.abc(instruction)?, i64::wrapping_mul)?
            }
            Opcode::DivideInteger => self.execute_divide_integer(self.abc(instruction)?)?,
            Opcode::RemainderInteger => self.execute_remainder_integer(self.abc(instruction)?)?,
            Opcode::AddReal => {
                self.execute_binary_real(self.abc(instruction)?, |left, right| left + right)?
            }
            Opcode::SubtractReal => {
                self.execute_binary_real(self.abc(instruction)?, |left, right| left - right)?
            }
            Opcode::MultiplyReal => {
                self.execute_binary_real(self.abc(instruction)?, |left, right| left * right)?
            }
            Opcode::DivideReal => self.execute_divide_real(self.abc(instruction)?)?,
            Opcode::NegateInteger => self.execute_negate_integer(self.abc(instruction)?)?,
            Opcode::NegateReal => self.execute_negate_real(self.abc(instruction)?)?,
            Opcode::AddDynamic => {
                self.execute_dynamic_arithmetic(self.abc(instruction)?, DynamicArithmetic::Add)?
            }
            Opcode::SubtractDynamic => self
                .execute_dynamic_arithmetic(self.abc(instruction)?, DynamicArithmetic::Subtract)?,
            Opcode::MultiplyDynamic => self
                .execute_dynamic_arithmetic(self.abc(instruction)?, DynamicArithmetic::Multiply)?,
            Opcode::DivideDynamic => self.execute_divide_dynamic(self.abc(instruction)?)?,
            Opcode::NegateDynamic => self.execute_negate_dynamic(self.abc(instruction)?)?,
            Opcode::EqualDynamic => self.execute_equal_dynamic(self.abc(instruction)?, true)?,
            Opcode::NotEqualDynamic => self.execute_equal_dynamic(self.abc(instruction)?, false)?,
            Opcode::LessDynamic => {
                self.execute_order_dynamic(self.abc(instruction)?, |ordering| ordering.is_lt())?
            }
            Opcode::GreaterDynamic => {
                self.execute_order_dynamic(self.abc(instruction)?, |ordering| ordering.is_gt())?
            }
            Opcode::LessEqualDynamic => {
                self.execute_order_dynamic(self.abc(instruction)?, |ordering| ordering.is_le())?
            }
            Opcode::GreaterEqualDynamic => {
                self.execute_order_dynamic(self.abc(instruction)?, |ordering| ordering.is_ge())?
            }
            Opcode::ConcatString => self.execute_concat_string(self.abc(instruction)?)?,
            Opcode::ShiftLeftInteger => self.execute_shift_integer(self.abc(instruction)?, true)?,
            Opcode::ShiftRightInteger => {
                self.execute_shift_integer(self.abc(instruction)?, false)?
            }
            Opcode::BitAndInteger => {
                self.execute_binary_integer(self.abc(instruction)?, |left, right| left & right)?
            }
            Opcode::BitOrInteger => {
                self.execute_binary_integer(self.abc(instruction)?, |left, right| left | right)?
            }
            Opcode::BitXorInteger => {
                self.execute_binary_integer(self.abc(instruction)?, |left, right| left ^ right)?
            }
            Opcode::EqualInteger => {
                self.execute_compare_integer(self.abc(instruction)?, |left, right| left == right)?
            }
            Opcode::NotEqualInteger => {
                self.execute_compare_integer(self.abc(instruction)?, |left, right| left != right)?
            }
            Opcode::LessInteger => {
                self.execute_compare_integer(self.abc(instruction)?, |left, right| left < right)?
            }
            Opcode::GreaterInteger => {
                self.execute_compare_integer(self.abc(instruction)?, |left, right| left > right)?
            }
            Opcode::LessEqualInteger => {
                self.execute_compare_integer(self.abc(instruction)?, |left, right| left <= right)?
            }
            Opcode::GreaterEqualInteger => {
                self.execute_compare_integer(self.abc(instruction)?, |left, right| left >= right)?
            }
            Opcode::EqualReal => {
                self.execute_compare_real(self.abc(instruction)?, |left, right| left == right)?
            }
            Opcode::NotEqualReal => {
                self.execute_compare_real(self.abc(instruction)?, |left, right| left != right)?
            }
            Opcode::LessReal => {
                self.execute_compare_real(self.abc(instruction)?, |left, right| left < right)?
            }
            Opcode::GreaterReal => {
                self.execute_compare_real(self.abc(instruction)?, |left, right| left > right)?
            }
            Opcode::LessEqualReal => {
                self.execute_compare_real(self.abc(instruction)?, |left, right| left <= right)?
            }
            Opcode::GreaterEqualReal => {
                self.execute_compare_real(self.abc(instruction)?, |left, right| left >= right)?
            }
            Opcode::EqualString => {
                self.execute_compare_string(self.abc(instruction)?, |ordering| ordering.is_eq())?
            }
            Opcode::NotEqualString => {
                self.execute_compare_string(self.abc(instruction)?, |ordering| !ordering.is_eq())?
            }
            Opcode::LessString => {
                self.execute_compare_string(self.abc(instruction)?, |ordering| ordering.is_lt())?
            }
            Opcode::GreaterString => {
                self.execute_compare_string(self.abc(instruction)?, |ordering| ordering.is_gt())?
            }
            Opcode::LessEqualString => {
                self.execute_compare_string(self.abc(instruction)?, |ordering| ordering.is_le())?
            }
            Opcode::GreaterEqualString => {
                self.execute_compare_string(self.abc(instruction)?, |ordering| ordering.is_ge())?
            }
            Opcode::EqualBoolean => {
                self.execute_compare_boolean(self.abc(instruction)?, |left, right| left == right)?
            }
            Opcode::NotEqualBoolean => {
                self.execute_compare_boolean(self.abc(instruction)?, |left, right| left != right)?
            }
            Opcode::NotBoolean => self.execute_not_boolean(self.abc(instruction)?)?,
            Opcode::AndBoolean => {
                self.execute_binary_boolean(self.abc(instruction)?, |left, right| left && right)?
            }
            Opcode::OrBoolean => {
                self.execute_binary_boolean(self.abc(instruction)?, |left, right| left || right)?
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
                return Ok(DispatchStep::Return(value));
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
            Opcode::CallDirect
            | Opcode::CallValue
            | Opcode::MakeClosure
            | Opcode::MakeCell
            | Opcode::CellRead
            | Opcode::CellWrite
            | Opcode::LoadGlobal
            | Opcode::StoreGlobal
            | Opcode::MakeArray
            | Opcode::IndexGet
            | Opcode::IndexSet
            | Opcode::Contains
            | Opcode::MakeDictionary
            | Opcode::MakeRecord
            | Opcode::LoadField
            | Opcode::StoreField
            | Opcode::UpdateRecord
            | Opcode::Intrinsic
            | Opcode::MakeOk
            | Opcode::MakeError
            | Opcode::MakeSome
            | Opcode::MakeNone
            | Opcode::IsResultOk
            | Opcode::IsOptionSome
            | Opcode::UnwrapOk
            | Opcode::UnwrapError
            | Opcode::UnwrapSome
            | Opcode::MakeEnum
            | Opcode::TestVariant
            | Opcode::LoadEnumField
            | Opcode::SpawnTask
            | Opcode::SpawnDetachedTask
            | Opcode::Yield
            | Opcode::ReservedMetadata => return Err(self.future_phase(opcode)),
        }
        Ok(DispatchStep::Continue)
    }

    fn abc(&self, instruction: fpas_bytecode::Instruction) -> Result<AbcOperands, VmError> {
        instruction.abc_operands().map_err(|error| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Verified ABC operands failed decoding: {error}"),
            )
        })
    }

    fn abx(&self, instruction: fpas_bytecode::Instruction) -> Result<AbxOperands, VmError> {
        instruction.abx_operands().map_err(|error| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Verified ABx operands failed decoding: {error}"),
            )
        })
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

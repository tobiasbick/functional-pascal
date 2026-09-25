//! Direct handlers for typed real, string, and Boolean bytecode.

use fpas_bytecode::{AbcOperands, Value};

use super::super::VmError;
use super::super::value_ops::{self, BinaryOperation};
use super::super::worker::Worker;

impl Worker {
    /// Execute real arithmetic without generic numeric dispatch when both operands are real.
    #[inline(always)]
    pub(in crate::vm) fn execute_real_binary(
        &mut self,
        operands: AbcOperands,
        operation: BinaryOperation,
    ) -> Result<(), VmError> {
        let left = self.read_operand(operands.b)?;
        let right = self.read_operand(operands.c)?;
        let result = match (left, right) {
            (Value::Real(left), Value::Real(right))
                if operation != BinaryOperation::RealDivide || *right != 0.0 =>
            {
                Value::Real(match operation {
                    BinaryOperation::Add => left + right,
                    BinaryOperation::Subtract => left - right,
                    BinaryOperation::Multiply => left * right,
                    BinaryOperation::RealDivide => left / right,
                    _ => unreachable!("real arithmetic opcode"),
                })
            }
            _ => value_ops::binary(operation, left, right)
                .map_err(|error| self.runtime_error(error.code, error.message, error.hint))?,
        };
        self.write_operand(operands.a, result)
    }

    /// Compare typed real operands directly while preserving generic NaN diagnostics.
    #[inline(always)]
    pub(in crate::vm) fn execute_real_comparison(
        &mut self,
        operands: AbcOperands,
        operation: BinaryOperation,
    ) -> Result<(), VmError> {
        let left = self.read_operand(operands.b)?;
        let right = self.read_operand(operands.c)?;
        let result = match (left, right) {
            (Value::Real(left), Value::Real(right)) if !left.is_nan() && !right.is_nan() => {
                Value::Boolean(compare(operation, *left, *right))
            }
            _ => value_ops::binary(operation, left, right)
                .map_err(|error| self.runtime_error(error.code, error.message, error.hint))?,
        };
        self.write_operand(operands.a, result)
    }

    /// Concatenate strings, growing the left buffer in place when the left register dies here.
    ///
    /// The compiler sets auxiliary 1 when the left operand is a temporary read for the last time;
    /// a destination equal to the left register qualifies as well. The destination's previous
    /// value is released first, so `S := S + X` leaves the copied buffer uniquely owned.
    #[inline(always)]
    pub(in crate::vm) fn execute_string_concat(
        &mut self,
        operands: AbcOperands,
    ) -> Result<(), VmError> {
        let consumes_left = operands.auxiliary == 1 || operands.a == operands.b;
        let in_place = consumes_left
            && operands.b != operands.c
            && matches!(self.read_operand(operands.b)?, Value::Str(_))
            && matches!(self.read_operand(operands.c)?, Value::Str(_));
        if !in_place {
            return self.execute_value_binary(operands, BinaryOperation::Add);
        }
        if operands.a != operands.b && operands.a != operands.c {
            // The destination is overwritten below; releasing it early can make `left` unique.
            self.take_register(self.base + usize::from(operands.a))?;
        }
        let Value::Str(mut left) = self.take_register(self.base + usize::from(operands.b))? else {
            unreachable!("left concatenation operand was checked as a string");
        };
        let Value::Str(right) = self.read_operand(operands.c)? else {
            unreachable!("right concatenation operand was checked as a string");
        };
        left.append(right);
        self.write_operand(operands.a, Value::Str(left))
    }

    /// Compare typed strings without generic value dispatch.
    #[inline(always)]
    pub(in crate::vm) fn execute_string_comparison(
        &mut self,
        operands: AbcOperands,
        operation: BinaryOperation,
    ) -> Result<(), VmError> {
        let left = self.read_operand(operands.b)?;
        let right = self.read_operand(operands.c)?;
        let result = match (left, right) {
            (Value::Str(left), Value::Str(right)) => {
                Value::Boolean(compare(operation, left.as_ref(), right.as_ref()))
            }
            _ => value_ops::binary(operation, left, right)
                .map_err(|error| self.runtime_error(error.code, error.message, error.hint))?,
        };
        self.write_operand(operands.a, result)
    }

    /// Execute typed Boolean equality and eager logical operators directly.
    #[inline(always)]
    pub(in crate::vm) fn execute_boolean_binary(
        &mut self,
        operands: AbcOperands,
        operation: BinaryOperation,
    ) -> Result<(), VmError> {
        let left = self.read_operand(operands.b)?;
        let right = self.read_operand(operands.c)?;
        let result = match (left, right) {
            (Value::Boolean(left), Value::Boolean(right)) => Value::Boolean(match operation {
                BinaryOperation::Equal => left == right,
                BinaryOperation::NotEqual => left != right,
                BinaryOperation::And => *left && *right,
                BinaryOperation::Or => *left || *right,
                _ => unreachable!("Boolean binary opcode"),
            }),
            _ => value_ops::binary(operation, left, right)
                .map_err(|error| self.runtime_error(error.code, error.message, error.hint))?,
        };
        self.write_operand(operands.a, result)
    }
}

#[inline(always)]
fn compare<T: PartialOrd + PartialEq>(operation: BinaryOperation, left: T, right: T) -> bool {
    match operation {
        BinaryOperation::Equal => left == right,
        BinaryOperation::NotEqual => left != right,
        BinaryOperation::Less => left < right,
        BinaryOperation::LessEqual => left <= right,
        BinaryOperation::Greater => left > right,
        BinaryOperation::GreaterEqual => left >= right,
        _ => unreachable!("typed comparison opcode"),
    }
}

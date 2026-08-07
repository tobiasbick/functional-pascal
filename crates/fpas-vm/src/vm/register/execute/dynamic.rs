//! Dynamically checked generic numeric and comparison handlers.

use std::cmp::Ordering;

use fpas_bytecode::{AbcOperands, Value};
use fpas_diagnostics::codes::{
    RUNTIME_DIVISION_BY_ZERO, RUNTIME_NUMERIC_DOMAIN_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::super::VmError;
use super::super::worker::RegisterWorker;
use super::scalar::register;

#[derive(Clone, Copy)]
pub(in crate::vm::register) enum DynamicArithmetic {
    Add,
    Subtract,
    Multiply,
}

impl RegisterWorker {
    pub fn execute_dynamic_arithmetic(
        &mut self,
        operands: AbcOperands,
        operation: DynamicArithmetic,
    ) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let result = dynamic_arithmetic(
            self.read(register(operands.b)?)?,
            self.read(register(operands.c)?)?,
            operation,
        )
        .ok_or_else(|| {
            self.runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                "Dynamic arithmetic requires numeric operands (integer or real)",
                "Ensure both operands are numeric types.",
            )
        })?;
        self.write(destination, result)
    }

    pub fn execute_divide_dynamic(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let left = numeric(self.read(register(operands.b)?)?);
        let right = numeric(self.read(register(operands.c)?)?);
        let (Some(left), Some(right)) = (left, right) else {
            return Err(self.runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                "Dynamic arithmetic requires numeric operands (integer or real)",
                "Ensure both operands are numeric types.",
            ));
        };
        if right == 0.0 {
            return Err(self.runtime_error(
                RUNTIME_DIVISION_BY_ZERO,
                "Division by zero",
                "Check the right-hand side before using `/`.",
            ));
        }
        self.write(destination, Value::Real(left / right))
    }

    pub fn execute_negate_dynamic(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let result = match self.read(register(operands.b)?)? {
            Value::Integer(value) => Value::Integer(value.checked_neg().ok_or_else(|| {
                self.runtime_error(
                    RUNTIME_NUMERIC_DOMAIN_ERROR,
                    "Integer negation overflow",
                    "Avoid negating the minimum integer value.",
                )
            })?),
            Value::Real(value) => Value::Real(-value),
            other => {
                return Err(self.runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "Cannot negate non-numeric value of type {}",
                        other.type_name()
                    ),
                    "Apply unary `-` only to integer or real values.",
                ));
            }
        };
        self.write(destination, result)
    }

    pub fn execute_equal_dynamic(
        &mut self,
        operands: AbcOperands,
        equal: bool,
    ) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let comparison = self.read(register(operands.b)?)? == self.read(register(operands.c)?)?;
        self.write(destination, Value::Boolean(comparison == equal))
    }

    pub fn execute_order_dynamic(
        &mut self,
        operands: AbcOperands,
        compare: impl FnOnce(Ordering) -> bool,
    ) -> Result<(), VmError> {
        let destination = register(operands.a)?;
        let ordering = dynamic_order(
            self.read(register(operands.b)?)?,
            self.read(register(operands.c)?)?,
        )
        .ok_or_else(|| {
            self.runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                "Dynamic ordered comparison requires scalar comparable operands",
                "Ensure both operands are integer, real, boolean, or string.",
            )
        })?;
        self.write(destination, Value::Boolean(compare(ordering)))
    }
}

fn dynamic_arithmetic(left: &Value, right: &Value, operation: DynamicArithmetic) -> Option<Value> {
    match (left, right) {
        (Value::Integer(left), Value::Integer(right)) => Some(Value::Integer(match operation {
            DynamicArithmetic::Add => left.wrapping_add(*right),
            DynamicArithmetic::Subtract => left.wrapping_sub(*right),
            DynamicArithmetic::Multiply => left.wrapping_mul(*right),
        })),
        _ => {
            let left = numeric(left)?;
            let right = numeric(right)?;
            Some(Value::Real(match operation {
                DynamicArithmetic::Add => left + right,
                DynamicArithmetic::Subtract => left - right,
                DynamicArithmetic::Multiply => left * right,
            }))
        }
    }
}

fn numeric(value: &Value) -> Option<f64> {
    match value {
        Value::Integer(value) => Some(*value as f64),
        Value::Real(value) => Some(*value),
        _ => None,
    }
}

fn dynamic_order(left: &Value, right: &Value) -> Option<Ordering> {
    match (left, right) {
        (Value::Integer(left), Value::Integer(right)) => Some(left.cmp(right)),
        (Value::Boolean(left), Value::Boolean(right)) => Some(left.cmp(right)),
        (Value::Str(left), Value::Str(right)) => Some(left.cmp(right)),
        _ => numeric(left)?.partial_cmp(&numeric(right)?),
    }
}

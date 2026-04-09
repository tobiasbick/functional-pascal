//! Polymorphic numeric and comparison ops for type-erased generics.
//!
//! These ops inspect runtime value types and dispatch to the correct operation.
//! Used when the compiler emits code for generic function bodies where the
//! concrete type is not known at compile time.
//!
//! **Documentation:** `docs/pascal/05-types.md` (Generics — Constraints)

use crate::vm::diagnostics::VmError;
use crate::vm::{Worker, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_DIVISION_BY_ZERO, RUNTIME_NUMERIC_DOMAIN_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

impl Worker {
    pub(super) fn try_exec_dynamic_ops(
        &mut self,
        op: Op,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match op {
            Op::AddDyn => {
                self.binary_numeric_dyn(line, |a, b| match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => Ok(Value::Integer(x.wrapping_add(y))),
                    (Value::Real(x), Value::Real(y)) => Ok(Value::Real(x + y)),
                    (Value::Integer(x), Value::Real(y)) => Ok(Value::Real(x as f64 + y)),
                    (Value::Real(x), Value::Integer(y)) => Ok(Value::Real(x + y as f64)),
                    _ => None.ok_or(()),
                })?;
                Ok(true)
            }
            Op::SubDyn => {
                self.binary_numeric_dyn(line, |a, b| match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => Ok(Value::Integer(x.wrapping_sub(y))),
                    (Value::Real(x), Value::Real(y)) => Ok(Value::Real(x - y)),
                    (Value::Integer(x), Value::Real(y)) => Ok(Value::Real(x as f64 - y)),
                    (Value::Real(x), Value::Integer(y)) => Ok(Value::Real(x - y as f64)),
                    _ => None.ok_or(()),
                })?;
                Ok(true)
            }
            Op::MulDyn => {
                self.binary_numeric_dyn(line, |a, b| match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => Ok(Value::Integer(x.wrapping_mul(y))),
                    (Value::Real(x), Value::Real(y)) => Ok(Value::Real(x * y)),
                    (Value::Integer(x), Value::Real(y)) => Ok(Value::Real(x as f64 * y)),
                    (Value::Real(x), Value::Integer(y)) => Ok(Value::Real(x * y as f64)),
                    _ => None.ok_or(()),
                })?;
                Ok(true)
            }
            Op::DivDyn => {
                let right = self.pop(line)?;
                let left = self.pop(line)?;
                let result = match (&left, &right) {
                    (Value::Integer(x), Value::Integer(y)) => {
                        if *y == 0 {
                            return Err(runtime_error(
                                RUNTIME_DIVISION_BY_ZERO,
                                "Division by zero",
                                "Check the right-hand side before using `/`.",
                                line,
                            ));
                        }
                        Ok(Value::Real(*x as f64 / *y as f64))
                    }
                    (Value::Real(_), Value::Real(y)) if *y == 0.0 => Err(runtime_error(
                        RUNTIME_DIVISION_BY_ZERO,
                        "Division by zero",
                        "Check the right-hand side before using `/`.",
                        line,
                    )),
                    (Value::Integer(_), Value::Real(y)) if *y == 0.0 => Err(runtime_error(
                        RUNTIME_DIVISION_BY_ZERO,
                        "Division by zero",
                        "Check the right-hand side before using `/`.",
                        line,
                    )),
                    (Value::Real(_), Value::Integer(y)) if *y == 0 => Err(runtime_error(
                        RUNTIME_DIVISION_BY_ZERO,
                        "Division by zero",
                        "Check the right-hand side before using `/`.",
                        line,
                    )),
                    (Value::Real(x), Value::Real(y)) => Ok(Value::Real(*x / *y)),
                    (Value::Integer(x), Value::Real(y)) => Ok(Value::Real(*x as f64 / *y)),
                    (Value::Real(x), Value::Integer(y)) => Ok(Value::Real(*x / *y as f64)),
                    _ => Err(runtime_error(
                        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                        "Dynamic arithmetic requires numeric operands (integer or real)",
                        "Ensure both operands are numeric types.",
                        line,
                    )),
                }?;
                self.push(result)?;
                Ok(true)
            }
            Op::NegateDyn => {
                let val = self.pop(line)?;
                match val {
                    Value::Integer(n) => {
                        let negated = n.checked_neg().ok_or_else(|| {
                            runtime_error(
                                RUNTIME_NUMERIC_DOMAIN_ERROR,
                                "Integer negation overflow",
                                "Avoid negating the minimum integer value.",
                                line,
                            )
                        })?;
                        self.push(Value::Integer(negated))?;
                    }
                    Value::Real(n) => self.push(Value::Real(-n))?,
                    _ => {
                        return Err(runtime_error(
                            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                            "Cannot negate non-numeric value",
                            "Apply unary `-` only to integer or real values.",
                            line,
                        ));
                    }
                }
                Ok(true)
            }
            Op::EqDyn => {
                self.binary_comparable_dyn(line, |ord| ord.is_eq())?;
                Ok(true)
            }
            Op::NeqDyn => {
                self.binary_comparable_dyn(line, |ord| !ord.is_eq())?;
                Ok(true)
            }
            Op::LtDyn => {
                self.binary_comparable_dyn(line, |ord| ord.is_lt())?;
                Ok(true)
            }
            Op::GtDyn => {
                self.binary_comparable_dyn(line, |ord| ord.is_gt())?;
                Ok(true)
            }
            Op::LeDyn => {
                self.binary_comparable_dyn(line, |ord| ord.is_le())?;
                Ok(true)
            }
            Op::GeDyn => {
                self.binary_comparable_dyn(line, |ord| ord.is_ge())?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn binary_numeric_dyn(
        &mut self,
        line: SourceLocation,
        f: impl FnOnce(Value, Value) -> Result<Value, ()>,
    ) -> Result<(), VmError> {
        let right = self.pop(line)?;
        let left = self.pop(line)?;
        match f(left, right) {
            Ok(result) => self.push(result),
            Err(()) => Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                "Dynamic arithmetic requires numeric operands (integer or real)",
                "Ensure both operands are numeric types.",
                line,
            )),
        }
    }

    fn binary_comparable_dyn(
        &mut self,
        line: SourceLocation,
        f: impl FnOnce(std::cmp::Ordering) -> bool,
    ) -> Result<(), VmError> {
        let right = self.pop(line)?;
        let left = self.pop(line)?;
        let ord = dyn_compare(&left, &right).ok_or_else(|| {
            runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                "Dynamic comparison requires comparable operands of compatible types",
                "Ensure both operands are comparable types (integer, real, boolean, char, string).",
                line,
            )
        })?;
        self.push(Value::Boolean(f(ord)))
    }
}

/// Compare two runtime values, returning `None` for incompatible or
/// unordered pairs (e.g. NaN).
fn dyn_compare(left: &Value, right: &Value) -> Option<std::cmp::Ordering> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Some(a.cmp(b)),
        (Value::Real(a), Value::Real(b)) => a.partial_cmp(b),
        (Value::Integer(a), Value::Real(b)) => (*a as f64).partial_cmp(b),
        (Value::Real(a), Value::Integer(b)) => a.partial_cmp(&(*b as f64)),
        (Value::Boolean(a), Value::Boolean(b)) => Some(a.cmp(b)),
        (Value::Char(a), Value::Char(b)) => Some(a.cmp(b)),
        (Value::Str(a), Value::Str(b)) => Some(a.cmp(b)),
        _ => None,
    }
}

//! Polymorphic numeric and comparison ops for type-erased generics.
//!
//! These ops inspect runtime value types and dispatch to the correct operation.
//! Used when the compiler emits code for generic function bodies where the
//! concrete type is not known at compile time.
//!
//! **Documentation:** `docs/pascal/05-types.md` (Generics — Constraints)

use super::super::super::diagnostics::VmError;
use super::super::super::{Worker, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::{RUNTIME_NUMERIC_DOMAIN_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH};

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
                self.binary_numeric_dyn(line, |a, b| match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => Ok(Value::Real(x as f64 / y as f64)),
                    (Value::Real(x), Value::Real(y)) => Ok(Value::Real(x / y)),
                    (Value::Integer(x), Value::Real(y)) => Ok(Value::Real(x as f64 / y)),
                    (Value::Real(x), Value::Integer(y)) => Ok(Value::Real(x / y as f64)),
                    _ => None.ok_or(()),
                })?;
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
                self.binary_comparable_dyn(line, |a, b| Value::Boolean(a == b))?;
                Ok(true)
            }
            Op::NeqDyn => {
                self.binary_comparable_dyn(line, |a, b| Value::Boolean(a != b))?;
                Ok(true)
            }
            Op::LtDyn => {
                self.binary_comparable_dyn(line, |a, b| Value::Boolean(a < b))?;
                Ok(true)
            }
            Op::GtDyn => {
                self.binary_comparable_dyn(line, |a, b| Value::Boolean(a > b))?;
                Ok(true)
            }
            Op::LeDyn => {
                self.binary_comparable_dyn(line, |a, b| Value::Boolean(a <= b))?;
                Ok(true)
            }
            Op::GeDyn => {
                self.binary_comparable_dyn(line, |a, b| Value::Boolean(a >= b))?;
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
        f: impl FnOnce(ComparableValue, ComparableValue) -> Value,
    ) -> Result<(), VmError> {
        let right = self.pop(line)?;
        let left = self.pop(line)?;
        match (
            ComparableValue::from_value(&left),
            ComparableValue::from_value(&right),
        ) {
            (Some(a), Some(b)) => self.push(f(a, b)),
            _ => Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                "Dynamic comparison requires comparable operands",
                "Ensure both operands are comparable types (integer, real, boolean, char, string).",
                line,
            )),
        }
    }
}

/// A wrapper that implements Ord/PartialOrd for runtime-polymorphic comparison.
#[derive(Debug, PartialEq)]
enum ComparableValue {
    Integer(i64),
    Real(f64),
    Boolean(bool),
    Char(char),
    Str(String),
}

impl ComparableValue {
    fn from_value(v: &Value) -> Option<Self> {
        match v {
            Value::Integer(n) => Some(Self::Integer(*n)),
            Value::Real(n) => Some(Self::Real(*n)),
            Value::Boolean(b) => Some(Self::Boolean(*b)),
            Value::Char(c) => Some(Self::Char(*c)),
            Value::Str(s) => Some(Self::Str(s.clone())),
            _ => None,
        }
    }

    fn discriminant_index(&self) -> u8 {
        match self {
            Self::Integer(_) => 0,
            Self::Real(_) => 1,
            Self::Boolean(_) => 2,
            Self::Char(_) => 3,
            Self::Str(_) => 4,
        }
    }
}

impl Eq for ComparableValue {}

impl PartialOrd for ComparableValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ComparableValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Integer(a), Self::Integer(b)) => a.cmp(b),
            (Self::Real(a), Self::Real(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            (Self::Integer(a), Self::Real(b)) => (*a as f64)
                .partial_cmp(b)
                .unwrap_or(std::cmp::Ordering::Equal),
            (Self::Real(a), Self::Integer(b)) => a
                .partial_cmp(&(*b as f64))
                .unwrap_or(std::cmp::Ordering::Equal),
            (Self::Boolean(a), Self::Boolean(b)) => a.cmp(b),
            (Self::Char(a), Self::Char(b)) => a.cmp(b),
            (Self::Str(a), Self::Str(b)) => a.cmp(b),
            // Cross-type comparison: use discriminant index as fallback.
            _ => self.discriminant_index().cmp(&other.discriminant_index()),
        }
    }
}

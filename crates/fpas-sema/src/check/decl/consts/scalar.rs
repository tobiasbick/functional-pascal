//! Scalar interface values for static expressions; failing operations remain runtime work.

use super::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_parser::{BinaryOp, Expr, UnaryOp};
use fpas_unit::interface::ConstantValue as Value;

impl Checker {
    /// Folds scalar static expressions using the language's runtime arithmetic rules.
    pub(crate) fn scalar_constant_value(&self, expr: &Expr) -> Option<Value> {
        match expr {
            Expr::Integer(value, _) => Some(Value::Integer(*value)),
            Expr::Real(value, _) => Some(Value::Real(value.to_bits())),
            Expr::Str(value, _) => Some(Value::String(value.clone())),
            Expr::Bool(value, _) => Some(Value::Boolean(*value)),
            Expr::Paren(inner, _) => self.scalar_constant_value(inner),
            Expr::Designator(designator) => {
                let name = Self::resolve_designator_name(designator);
                let symbol = self.scopes.lookup(&name)?;
                if let Some(info) = &symbol.constant {
                    return info.value.clone();
                }
                if symbol.kind == SymbolKind::EnumMember
                    && let Ty::Enum(enum_ty) = &symbol.ty
                {
                    let short = name.rsplit('.').next()?;
                    let variant = enum_ty
                        .variants
                        .iter()
                        .find(|variant| variant.name.eq_ignore_ascii_case(short))?;
                    return Some(Value::EnumValue {
                        enum_name: enum_ty.name.clone(),
                        variant_name: variant.name.clone(),
                        backing_value: variant.backing_value?,
                    });
                }
                None
            }
            Expr::UnaryOp { op, operand, .. } => match (op, self.scalar_constant_value(operand)?) {
                (UnaryOp::Negate, Value::Integer(value)) => value.checked_neg().map(Value::Integer),
                (UnaryOp::Negate, Value::Real(bits)) => {
                    Some(Value::Real((-f64::from_bits(bits)).to_bits()))
                }
                (UnaryOp::Not, Value::Boolean(value)) => Some(Value::Boolean(!value)),
                _ => None,
            },
            Expr::BinaryOp {
                op, left, right, ..
            } => {
                let left = self.scalar_constant_value(left)?;
                // Preserve short-circuit behavior even if the unused operand would fail.
                if matches!((op, &left), (BinaryOp::And, Value::Boolean(false))) {
                    return Some(Value::Boolean(false));
                }
                if matches!((op, &left), (BinaryOp::Or, Value::Boolean(true))) {
                    return Some(Value::Boolean(true));
                }
                binary(*op, left, self.scalar_constant_value(right)?)
            }
            _ => None,
        }
    }
}

fn binary(op: BinaryOp, left: Value, right: Value) -> Option<Value> {
    use BinaryOp as Op;
    use Value::{Boolean, Integer, Real, String};
    match (op, left, right) {
        (Op::Add, Integer(a), Integer(b)) => Some(Integer(a.wrapping_add(b))),
        (Op::Sub, Integer(a), Integer(b)) => Some(Integer(a.wrapping_sub(b))),
        (Op::Mul, Integer(a), Integer(b)) => Some(Integer(a.wrapping_mul(b))),
        (Op::IntDiv, Integer(a), Integer(b)) => a.checked_div(b).map(Integer),
        (Op::Mod, Integer(a), Integer(b)) => a.checked_rem(b).map(Integer),
        (Op::RealDiv, Integer(a), Integer(b)) => real_binary(op, a as f64, b as f64),
        (op, Real(a), Real(b)) => real_binary(op, f64::from_bits(a), f64::from_bits(b)),
        (op, Real(a), Integer(b)) => real_binary(op, f64::from_bits(a), b as f64),
        (op, Integer(a), Real(b)) => real_binary(op, a as f64, f64::from_bits(b)),
        (Op::Add, String(a), String(b)) => Some(String(a + &b)),
        (Op::And, Boolean(a), Boolean(b)) => Some(Boolean(a && b)),
        (Op::Or, Boolean(a), Boolean(b)) => Some(Boolean(a || b)),
        (Op::Xor, Boolean(a), Boolean(b)) => Some(Boolean(a ^ b)),
        (op, a, b) => {
            let order = match (&a, &b) {
                (Integer(a), Integer(b)) => a.partial_cmp(b),
                (String(a), String(b)) => a.partial_cmp(b),
                (Boolean(a), Boolean(b)) => a.partial_cmp(b),
                (
                    Value::EnumValue {
                        enum_name: an,
                        backing_value: a,
                        ..
                    },
                    Value::EnumValue {
                        enum_name: bn,
                        backing_value: b,
                        ..
                    },
                ) if an.eq_ignore_ascii_case(bn) => a.partial_cmp(b),
                _ => return None,
            };
            compare(op, order).map(Boolean)
        }
    }
}

fn real_binary(op: BinaryOp, a: f64, b: f64) -> Option<Value> {
    let result = match op {
        BinaryOp::Add => a + b,
        BinaryOp::Sub => a - b,
        BinaryOp::Mul => a * b,
        BinaryOp::RealDiv if b != 0.0 => a / b,
        _ => return compare(op, a.partial_cmp(&b)).map(Value::Boolean),
    };
    Some(Value::Real(result.to_bits()))
}

fn compare(op: BinaryOp, order: Option<std::cmp::Ordering>) -> Option<bool> {
    use std::cmp::Ordering::{Equal, Greater, Less};
    Some(match op {
        BinaryOp::Eq => order == Some(Equal),
        BinaryOp::NotEq => order != Some(Equal),
        BinaryOp::Lt => order == Some(Less),
        BinaryOp::Gt => order == Some(Greater),
        BinaryOp::LtEq => matches!(order, Some(Less | Equal)),
        BinaryOp::GtEq => matches!(order, Some(Greater | Equal)),
        _ => return None,
    })
}

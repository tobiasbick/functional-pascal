//! Constant folding: replace operations on constant operands by their constant result.
//!
//! Folding follows the VM's runtime semantics exactly: integer `+`, `-`, and `*` wrap, and any
//! operation that would raise a runtime error (division or modulo by zero, integer overflow in
//! `div`, `mod`, or negation, real division by zero) is left in place so the error still occurs.
//! A folded instruction keeps its index, so debugger sequence points stay valid.

use std::collections::BTreeMap;

use fpas_ir::{BinaryOperation, Constant, Function, Operation, UnaryOperation, ValueId};

/// Fold every foldable operation of `function` in place.
pub(super) fn fold_constants(function: &mut Function) {
    let mut constants = BTreeMap::<ValueId, Constant>::new();
    for block in &mut function.blocks {
        for instruction in &mut block.instructions {
            let folded = match &instruction.operation {
                Operation::Binary {
                    operation,
                    left,
                    right,
                } => constants
                    .get(left)
                    .zip(constants.get(right))
                    .and_then(|(left, right)| fold_binary(*operation, left, right)),
                Operation::Unary { operation, operand } => constants
                    .get(operand)
                    .and_then(|operand| fold_unary(*operation, operand)),
                _ => None,
            };
            if let Some(folded) = folded {
                instruction.operation = Operation::Const(folded);
            }
            if let (Operation::Const(constant), Some(result)) =
                (&instruction.operation, instruction.result)
            {
                constants.insert(result.id, constant.clone());
            }
        }
    }
}

fn fold_binary(operation: BinaryOperation, left: &Constant, right: &Constant) -> Option<Constant> {
    use BinaryOperation as Op;
    use Constant::{Boolean, Integer, Real, String};
    Some(match (operation, left, right) {
        (Op::AddInteger, Integer(left), Integer(right)) => Integer(left.wrapping_add(*right)),
        (Op::SubtractInteger, Integer(left), Integer(right)) => Integer(left.wrapping_sub(*right)),
        (Op::MultiplyInteger, Integer(left), Integer(right)) => Integer(left.wrapping_mul(*right)),
        (Op::DivideInteger, Integer(left), Integer(right)) => Integer(left.checked_div(*right)?),
        (Op::RemainderInteger, Integer(left), Integer(right)) => Integer(left.checked_rem(*right)?),
        (Op::BitAndInteger, Integer(left), Integer(right)) => Integer(left & right),
        (Op::BitOrInteger, Integer(left), Integer(right)) => Integer(left | right),
        (Op::BitXorInteger, Integer(left), Integer(right)) => Integer(left ^ right),
        (Op::LessThanInteger, Integer(left), Integer(right)) => Boolean(left < right),
        (Op::GreaterThanInteger, Integer(left), Integer(right)) => Boolean(left > right),
        (Op::LessEqualInteger, Integer(left), Integer(right)) => Boolean(left <= right),
        (Op::GreaterEqualInteger, Integer(left), Integer(right)) => Boolean(left >= right),
        (Op::AddReal, Real(left), Real(right)) => Real(left + right),
        (Op::SubtractReal, Real(left), Real(right)) => Real(left - right),
        (Op::MultiplyReal, Real(left), Real(right)) => Real(left * right),
        (Op::DivideReal, Real(left), Real(right)) if *right != 0.0 => Real(left / right),
        (Op::Equal | Op::NotEqual, left, right) => {
            let equal = match (left, right) {
                (Integer(left), Integer(right)) => left == right,
                (Real(left), Real(right)) => left == right,
                (Boolean(left), Boolean(right)) => left == right,
                (String(left), String(right)) => left == right,
                _ => return None,
            };
            Boolean(equal == (operation == Op::Equal))
        }
        (Op::AndBoolean, Boolean(left), Boolean(right)) => Boolean(*left && *right),
        (Op::OrBoolean, Boolean(left), Boolean(right)) => Boolean(*left || *right),
        (Op::ConcatString, String(left), String(right)) => String(format!("{left}{right}")),
        _ => return None,
    })
}

fn fold_unary(operation: UnaryOperation, operand: &Constant) -> Option<Constant> {
    Some(match (operation, operand) {
        (UnaryOperation::NegateInteger, Constant::Integer(value)) => {
            Constant::Integer(value.checked_neg()?)
        }
        (UnaryOperation::NegateReal, Constant::Real(value)) => Constant::Real(-value),
        (UnaryOperation::NotBoolean, Constant::Boolean(value)) => Constant::Boolean(!value),
        (UnaryOperation::IntegerToReal, Constant::Integer(value)) => Constant::Real(*value as f64),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folding_matches_runtime_semantics_and_keeps_failing_operations() {
        use Constant::{Boolean, Integer, Real, String};
        assert_eq!(
            fold_binary(BinaryOperation::AddInteger, &Integer(i64::MAX), &Integer(1)),
            Some(Integer(i64::MIN)),
            "integer addition wraps like the VM"
        );
        assert_eq!(
            fold_binary(BinaryOperation::DivideInteger, &Integer(7), &Integer(0)),
            None
        );
        assert_eq!(
            fold_binary(
                BinaryOperation::DivideInteger,
                &Integer(i64::MIN),
                &Integer(-1)
            ),
            None
        );
        assert_eq!(
            fold_binary(BinaryOperation::RemainderInteger, &Integer(-7), &Integer(2)),
            Some(Integer(-1))
        );
        assert_eq!(
            fold_binary(BinaryOperation::DivideReal, &Real(1.0), &Real(0.0)),
            None
        );
        assert_eq!(
            fold_binary(
                BinaryOperation::ConcatString,
                &String("a".to_string()),
                &String("b".to_string())
            ),
            Some(String("ab".to_string()))
        );
        assert_eq!(
            fold_binary(BinaryOperation::Equal, &Integer(1), &Real(1.0)),
            None
        );
        assert_eq!(
            fold_unary(UnaryOperation::NegateInteger, &Integer(i64::MIN)),
            None
        );
        assert_eq!(
            fold_unary(UnaryOperation::NotBoolean, &Boolean(true)),
            Some(Boolean(false))
        );
    }
}

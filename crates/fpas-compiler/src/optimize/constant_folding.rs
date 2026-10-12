//! Constant folding: replace operations on constant operands by their constant result.
//!
//! Folding uses the shared checked numeric operations. Integer overflow, zero divisors, and
//! nonfinite real results remain instructions so their errors occur on the executed path.
//! A folded instruction keeps its index, so debugger sequence points stay valid.
//! Boolean short-circuiting is already lowered to branches before this pass; folding an
//! instruction never evaluates it or moves it out of its guarded block.
//! Documentation: `docs/pascal/language/basics/operators.md#evaluation-order`.

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
        (Op::AddInteger, Integer(left), Integer(right)) => {
            integer_binary(fpas_numeric::integer::Operation::Add, *left, *right)?
        }
        (Op::SubtractInteger, Integer(left), Integer(right)) => {
            integer_binary(fpas_numeric::integer::Operation::Subtract, *left, *right)?
        }
        (Op::MultiplyInteger, Integer(left), Integer(right)) => {
            integer_binary(fpas_numeric::integer::Operation::Multiply, *left, *right)?
        }
        (Op::DivideInteger, Integer(left), Integer(right)) => {
            integer_binary(fpas_numeric::integer::Operation::Divide, *left, *right)?
        }
        (Op::RemainderInteger, Integer(left), Integer(right)) => {
            integer_binary(fpas_numeric::integer::Operation::Remainder, *left, *right)?
        }
        (Op::LessThanInteger, Integer(left), Integer(right)) => Boolean(left < right),
        (Op::GreaterThanInteger, Integer(left), Integer(right)) => Boolean(left > right),
        (Op::LessEqualInteger, Integer(left), Integer(right)) => Boolean(left <= right),
        (Op::GreaterEqualInteger, Integer(left), Integer(right)) => Boolean(left >= right),
        (Op::AddReal, Real(left), Real(right)) => {
            real_binary(fpas_numeric::real::Operation::Add, *left, *right)?
        }
        (Op::SubtractReal, Real(left), Real(right)) => {
            real_binary(fpas_numeric::real::Operation::Subtract, *left, *right)?
        }
        (Op::MultiplyReal, Real(left), Real(right)) => {
            real_binary(fpas_numeric::real::Operation::Multiply, *left, *right)?
        }
        (Op::DivideReal, Real(left), Real(right)) => {
            real_binary(fpas_numeric::real::Operation::Divide, *left, *right)?
        }
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
            Constant::Integer(fpas_numeric::integer::negate(*value).ok()?)
        }
        (UnaryOperation::NegateReal, Constant::Real(value)) => {
            Constant::Real(fpas_numeric::real::finite(-value).ok()?)
        }
        (UnaryOperation::NotBoolean, Constant::Boolean(value)) => Constant::Boolean(!value),
        (UnaryOperation::IntegerToReal, Constant::Integer(value)) => Constant::Real(*value as f64),
        _ => return None,
    })
}

fn integer_binary(
    operation: fpas_numeric::integer::Operation,
    left: i64,
    right: i64,
) -> Option<Constant> {
    fpas_numeric::integer::binary(operation, left, right)
        .ok()
        .map(Constant::Integer)
}

fn real_binary(
    operation: fpas_numeric::real::Operation,
    left: f64,
    right: f64,
) -> Option<Constant> {
    fpas_numeric::real::binary(operation, left, right)
        .ok()
        .map(Constant::Real)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folding_matches_runtime_semantics_and_keeps_failing_operations() {
        use Constant::{Boolean, Integer, Real, String};
        assert_eq!(
            fold_binary(BinaryOperation::AddInteger, &Integer(i64::MAX), &Integer(1)),
            None,
            "integer overflow must remain an executed instruction"
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

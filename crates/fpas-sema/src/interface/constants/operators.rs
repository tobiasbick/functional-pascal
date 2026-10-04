//! Scalar operator selection shared by static expression evaluation.

use fpas_ir::{BinaryOperation as Op, Constant};
use fpas_parser::BinaryOp;

/// Apply the ordinary numeric promotion used by scalar operations.
pub(super) fn promote_real(value: &mut Constant) {
    if let Constant::Integer(integer) = value {
        *value = Constant::Real(*integer as f64);
    }
}

/// Map a checked AST operator to its scalar IR operation.
pub(super) fn binary_operation(operation: BinaryOp, left: &Constant) -> Option<Op> {
    use BinaryOp as Ast;
    Some(match (operation, left) {
        (Ast::Eq, _) => Op::Equal,
        (Ast::NotEq | Ast::Xor, _) => Op::NotEqual,
        (Ast::And, _) => Op::AndBoolean,
        (Ast::Or, _) => Op::OrBoolean,
        (Ast::Add, Constant::String(_)) => Op::ConcatString,
        (Ast::Add, Constant::Integer(_)) => Op::AddInteger,
        (Ast::Sub, Constant::Integer(_)) => Op::SubtractInteger,
        (Ast::Mul, Constant::Integer(_)) => Op::MultiplyInteger,
        (Ast::IntDiv, _) => Op::DivideInteger,
        (Ast::Mod, _) => Op::RemainderInteger,
        (Ast::Add, Constant::Real(_)) => Op::AddReal,
        (Ast::Sub, Constant::Real(_)) => Op::SubtractReal,
        (Ast::Mul, Constant::Real(_)) => Op::MultiplyReal,
        (Ast::RealDiv, _) => Op::DivideReal,
        (Ast::Lt, Constant::Integer(_)) => Op::LessThanInteger,
        (Ast::Gt, Constant::Integer(_)) => Op::GreaterThanInteger,
        (Ast::LtEq, Constant::Integer(_)) => Op::LessEqualInteger,
        (Ast::GtEq, Constant::Integer(_)) => Op::GreaterEqualInteger,
        (Ast::Lt, Constant::Real(_)) => Op::LessThanReal,
        (Ast::Gt, Constant::Real(_)) => Op::GreaterThanReal,
        (Ast::LtEq, Constant::Real(_)) => Op::LessEqualReal,
        (Ast::GtEq, Constant::Real(_)) => Op::GreaterEqualReal,
        (Ast::Lt, Constant::String(_)) => Op::LessThanString,
        (Ast::Gt, Constant::String(_)) => Op::GreaterThanString,
        (Ast::LtEq, Constant::String(_)) => Op::LessEqualString,
        (Ast::GtEq, Constant::String(_)) => Op::GreaterEqualString,
        _ => return None,
    })
}

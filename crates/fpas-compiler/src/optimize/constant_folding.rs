//! Constant folding: replace operations on constant operands by their constant result.
//!
//! Folding follows the VM's runtime semantics exactly: integer `+`, `-`, and `*` wrap, and any
//! operation that would raise a runtime error (division or modulo by zero, integer overflow in
//! `div`, `mod`, or negation, real division by zero) is left in place so the error still occurs.
//! A folded instruction keeps its index, so debugger sequence points stay valid.

use std::collections::BTreeMap;

use fpas_ir::constants::{fold_binary, fold_unary};
use fpas_ir::{Constant, Function, Operation, ValueId};

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

//! Basic-block, terminator, target, and reachability validation.

use std::collections::BTreeSet;

use crate::{BasicBlock, BlockId, Function};

use super::{EntityKind, ValidationError, ValidationErrorKind, function_error};

/// Validates structural control-flow invariants for one function.
pub(crate) fn validate_function(function: &Function) -> Result<(), ValidationError> {
    let mut blocks = BTreeSet::new();
    for block in &function.blocks {
        if !blocks.insert(block.id) {
            return Err(function_error(
                function.id,
                Some(block.id),
                None,
                ValidationErrorKind::DuplicateId {
                    entity: EntityKind::Block,
                    id: block.id.get(),
                },
            ));
        }
        validate_terminators(function, block)?;
    }
    if !blocks.contains(&function.entry) {
        return Err(function_error(
            function.id,
            None,
            None,
            ValidationErrorKind::UnknownId {
                entity: EntityKind::Block,
                id: function.entry.get(),
            },
        ));
    }
    validate_targets(function, &blocks)?;
    validate_reachability(function, &blocks)
}

fn validate_terminators(function: &Function, block: &BasicBlock) -> Result<(), ValidationError> {
    match block.terminators.len() {
        0 => Err(function_error(
            function.id,
            Some(block.id),
            None,
            ValidationErrorKind::MissingTerminator,
        )),
        1 => Ok(()),
        count => Err(function_error(
            function.id,
            Some(block.id),
            None,
            ValidationErrorKind::MultipleTerminators { count },
        )),
    }
}

fn validate_targets(
    function: &Function,
    blocks: &BTreeSet<BlockId>,
) -> Result<(), ValidationError> {
    for block in &function.blocks {
        let Some(terminator) = block.terminators.first() else {
            continue;
        };
        for target in terminator.targets() {
            if !blocks.contains(&target.block) {
                return Err(function_error(
                    function.id,
                    Some(block.id),
                    None,
                    ValidationErrorKind::UnknownId {
                        entity: EntityKind::Block,
                        id: target.block.get(),
                    },
                ));
            }
        }
    }
    Ok(())
}

fn validate_reachability(
    function: &Function,
    blocks: &BTreeSet<BlockId>,
) -> Result<(), ValidationError> {
    let mut pending = vec![function.entry];
    let mut reached = BTreeSet::new();
    while let Some(block_id) = pending.pop() {
        if !reached.insert(block_id) {
            continue;
        }
        let Some(block) = function.block(block_id) else {
            continue;
        };
        let Some(terminator) = block.terminators.first() else {
            continue;
        };
        for target in terminator.targets() {
            pending.push(target.block);
        }
    }
    for block in blocks {
        if !reached.contains(block) {
            return Err(function_error(
                function.id,
                Some(*block),
                None,
                ValidationErrorKind::UnreachableBlock { block: *block },
            ));
        }
    }
    Ok(())
}

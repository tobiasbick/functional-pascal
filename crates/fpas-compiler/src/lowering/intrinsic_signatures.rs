//! Shared intrinsic signatures collected from all typed call sites.
//!
//! Polymorphic task results: `docs/pascal/std/concurrency/task.md`.

use std::collections::BTreeMap;

use fpas_ir::{Function, IntrinsicId, IntrinsicSignature, IrType, Operation, TypeId};

use super::types;

/// Collect signatures without letting the first call fix a polymorphic result to Unit.
pub(super) fn collect_intrinsic_signatures(
    functions: &[Function],
    types: &types::TypeTable,
) -> Vec<IntrinsicSignature> {
    let mut shapes = BTreeMap::<IntrinsicId, (usize, TypeId)>::new();
    for function in functions {
        for instruction in function.blocks.iter().flat_map(|block| &block.instructions) {
            if let Operation::Intrinsic {
                intrinsic,
                arguments,
            } = &instruction.operation
                && let Some(result) = instruction.result
            {
                let result = if matches!(types.kind(result.ty), Some(IrType::Unit)) {
                    types::UNIT
                } else {
                    types::DYNAMIC
                };
                shapes
                    .entry(*intrinsic)
                    .and_modify(|(_, shared_result)| {
                        if *shared_result != result {
                            *shared_result = types::DYNAMIC;
                        }
                    })
                    .or_insert((arguments.len(), result));
            }
        }
    }
    shapes
        .into_iter()
        .map(|(id, (arity, result))| {
            let wire = u16::try_from(id.get()).ok();
            let variadic = wire.and_then(fpas_bytecode::Intrinsic::from_u16)
                == Some(fpas_bytecode::Intrinsic::Str(
                    fpas_bytecode::StrIntrinsic::Format,
                ));
            let parameters = if variadic {
                vec![types::DYNAMIC, types::DYNAMIC]
            } else {
                vec![types::DYNAMIC; arity]
            };
            IntrinsicSignature {
                id,
                parameters,
                variadic,
                result,
            }
        })
        .collect()
}

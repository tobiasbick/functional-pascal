//! Reachable block ordering for the dense construction graph.

use fpas_ir::{BasicBlock, BlockId};

/// Orders reachable blocks without recursion; construction IDs must equal vector indices.
pub(super) fn reverse_postorder(blocks: &[BasicBlock], entry: BlockId) -> Vec<BasicBlock> {
    let mut seen = vec![false; blocks.len()];
    let mut postorder = Vec::new();
    let mut pending = vec![(entry, false)];
    while let Some((id, exiting)) = pending.pop() {
        if exiting {
            postorder.push(id);
            continue;
        }
        let index = id.get() as usize;
        let Some(block) = blocks.get(index).filter(|block| block.id == id) else {
            continue;
        };
        if std::mem::replace(&mut seen[index], true) {
            continue;
        }
        pending.push((id, true));
        if let Some(terminator) = block.terminators.first() {
            // LIFO visits successors in reverse source order before the final reversal.
            pending.extend(
                terminator
                    .targets()
                    .into_iter()
                    .map(|target| (target.block, false)),
            );
        }
    }
    postorder
        .into_iter()
        .rev()
        .filter_map(|id| blocks.get(id.get() as usize).cloned())
        .collect()
}

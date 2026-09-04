//! Reachability, deterministic successor order, and cycles in lowered graphs.

use fpas_ir::{BasicBlock, BlockId, Terminator, ValueId};

use super::block_order::reverse_postorder;
use super::{empty_block, target};

fn block(id: u32, terminator: Terminator) -> BasicBlock {
    let mut block = empty_block(BlockId::new(id));
    block.terminators.push(terminator);
    block
}

fn jump(id: u32) -> Terminator {
    Terminator::Jump(target(BlockId::new(id)))
}

fn branch(then_id: u32, else_id: u32) -> Terminator {
    Terminator::Branch {
        condition: ValueId::new(0),
        then_target: target(BlockId::new(then_id)),
        else_target: target(BlockId::new(else_id)),
    }
}

#[test]
fn reverse_postorder_preserves_successor_order_and_omits_unreachable_blocks() {
    let blocks = [
        block(0, branch(3, 1)),
        block(1, jump(2)),
        block(2, jump(4)),
        block(3, jump(4)),
        block(4, Terminator::Return(None)),
        block(5, jump(0)),
    ];
    let ordered = reverse_postorder(&blocks, BlockId::new(0));
    assert_eq!(
        ordered
            .iter()
            .map(|block| block.id.get())
            .collect::<Vec<_>>(),
        [0, 3, 1, 2, 4]
    );
}

#[test]
fn reverse_postorder_handles_a_loop_back_edge_once() {
    let blocks = [
        block(0, jump(1)),
        block(1, branch(2, 3)),
        block(2, jump(1)),
        block(3, Terminator::Return(None)),
    ];
    let ordered = reverse_postorder(&blocks, BlockId::new(0));
    assert_eq!(
        ordered
            .iter()
            .map(|block| block.id.get())
            .collect::<Vec<_>>(),
        [0, 1, 2, 3]
    );
}

#[test]
fn reverse_postorder_skips_missing_blocks_without_panicking() {
    assert!(reverse_postorder(&[], BlockId::new(0)).is_empty());
    let blocks = [block(0, jump(99))];
    assert_eq!(reverse_postorder(&blocks, BlockId::new(0)), blocks);
    assert!(reverse_postorder(&blocks, BlockId::new(99)).is_empty());
}

#[test]
fn reverse_postorder_handles_a_deep_graph_on_a_small_thread_stack() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let blocks = (0..10_000)
                .map(|id| {
                    block(
                        id,
                        if id == 9_999 {
                            Terminator::Return(None)
                        } else {
                            jump(id + 1)
                        },
                    )
                })
                .collect::<Vec<_>>();
            let ordered = reverse_postorder(&blocks, BlockId::new(0));
            assert_eq!(ordered, blocks);
        })
        .expect("small-stack traversal thread")
        .join()
        .expect("deep graph traversal");
}

//! Expected worker count for spawn bytecode (mirrors [`crate::vm::Vm::build`]).
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md` (Phase 4 checklist), `docs/pascal/language/concurrency/README.md`

pub(crate) fn expected_spawn_pool_workers() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .saturating_sub(1)
        .max(1)
}

//! Cooperative concurrency: task spawning, yielding, and scheduling.
//!
//! **Documentation:** `docs/pascal/08-concurrency.md`, `docs/future/parallel-vm.md`

mod tasks;

use super::super::Worker;
use super::super::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, Op, SourceLocation};

impl Worker {
    /// Handle concurrency opcodes: `SpawnTask`, `Yield`.
    pub(super) fn try_exec_concurrency(
        &mut self,
        op: Op,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match op {
            Op::SpawnTask(argc) => {
                self.exec_spawn_task(argc, true, line)?;
                Ok(true)
            }
            Op::SpawnDetachedTask(argc) => {
                self.exec_spawn_task(argc, false, line)?;
                Ok(true)
            }
            Op::Yield => {
                self.exec_yield();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Handle concurrency-related intrinsics. Returns `true` if handled.
    pub(super) fn try_exec_concurrency_intrinsic(
        &mut self,
        intr: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intr {
            Intrinsic::TaskWait => {
                self.exec_task_wait(line)?;
                Ok(true)
            }
            Intrinsic::TaskWaitAll => {
                self.exec_task_wait_all(line)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

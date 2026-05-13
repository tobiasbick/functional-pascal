//! Cooperative concurrency: task spawning, yielding, and scheduling.
//!
//! **Documentation:** `docs/pascal/08-concurrency.md`, `docs/rust/parallel-vm.md`

mod tasks;

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, Op, SourceLocation, TaskIntrinsic};

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
            Intrinsic::Task(TaskIntrinsic::Wait) => {
                self.exec_task_wait(line)?;
                Ok(true)
            }
            Intrinsic::Task(TaskIntrinsic::WaitAll) => {
                self.exec_task_wait_all(line)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

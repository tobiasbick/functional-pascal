//! Cooperative concurrency: task spawning, yielding, channels, and scheduling.
//!
//! **Documentation:** `docs/pascal/08-concurrency.md`, `docs/future/parallel-vm.md`

mod channels;
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
            Intrinsic::ChannelMake => {
                self.exec_channel_make(1, line)?;
                Ok(true)
            }
            Intrinsic::ChannelMakeBuffered => {
                let size = self.pop_int(line)?;
                self.exec_channel_make_buffered(size, line)?;
                Ok(true)
            }
            Intrinsic::ChannelSend => {
                self.exec_channel_send(line)?;
                Ok(true)
            }
            Intrinsic::ChannelRecv => {
                self.exec_channel_recv(line)?;
                Ok(true)
            }
            Intrinsic::ChannelTryRecv => {
                self.exec_channel_try_recv(line)?;
                Ok(true)
            }
            Intrinsic::ChannelClose => {
                self.exec_channel_close(line)?;
                Ok(true)
            }
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

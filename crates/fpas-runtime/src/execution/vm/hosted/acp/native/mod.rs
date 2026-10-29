//! Owned ACP host effects keep conversion on the VM and retirement outside host callbacks.

mod call;
mod host;
mod output;
#[cfg(test)]
mod tests;
pub(in crate::execution::vm) mod wait;

use crate::AcpIntrinsic as Op;
pub(in crate::execution::vm) fn supported(op: Op) -> bool {
    matches!(
        op,
        Op::ConnectMessages
            | Op::ConnectProcess
            | Op::HostReadTextFile
            | Op::HostWriteTextFile
            | Op::HostCreateTerminal
            | Op::HostTerminalOutput
            | Op::HostWaitForTerminalExit
            | Op::HostKillTerminal
            | Op::HostReleaseTerminal
            | Op::HostRunTerminalAuthentication
            | Op::HostV2RunTerminalAuthentication
    )
}

impl crate::execution::vm::worker::Worker {
    pub(in super::super) fn execute_acp_native(
        &self,
        op: Op,
        values: &[crate::Value],
        location: crate::SourceLocation,
    ) -> Result<crate::Value, crate::execution::vm::VmError> {
        let result = call::Call::prepare(self, op, values).and_then(|call| {
            call.run(&self.hosted, || self.check_task_cancellation().is_err())?
                .into_value(self, location)
        });
        self.acp_result(&op.source_name(), result, location)
    }
}

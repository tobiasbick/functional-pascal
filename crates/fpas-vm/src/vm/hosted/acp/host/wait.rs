use super::{member, operation, raw};
use crate::vm::{hosted::proc::AcpTerminals, worker::Worker};
use fpas_acp::{
    connection::{Fault, HostClaim},
    protocol::v1 as sdk,
};
use fpas_bytecode::{SourceLocation, Value};
use std::sync::Arc;

pub(in crate::vm) struct TerminalWait {
    pub token: Option<u64>,
    pub deadline: Option<u64>,
    terminals: Arc<AcpTerminals>,
    session: String,
    id: String,
    _claim: HostClaim,
}
impl Worker {
    pub(in super::super) fn acp_terminal_wait(
        &self,
        arguments: &[Value],
    ) -> Result<TerminalWait, Fault> {
        use super::super::arguments as args;
        let [owner, payload, timeout, cancellation] = arguments else {
            return Err(Fault::InvalidArgument("Arguments"));
        };
        let timeout = args::timeout(timeout)?;
        let token = args::optional(cancellation)?
            .map(args::handle)
            .transpose()?;
        if let Some(token) = token {
            if self
                .hosted
                .cancellations
                .is_cancelled(token)
                .map_err(|_| Fault::InvalidArgument("CancellationToken"))?
            {
                return Err(Fault::Cancelled);
            }
        }
        let (connection, responder) = self.hosted.acp.responder(args::handle(owner)?)?;
        let (method, key, _) = operation(fpas_bytecode::AcpIntrinsic::HostWaitForTerminalExit)
            .ok_or(Fault::InvalidArgument("Operation"))?;
        let request = self.acp_host_request(key, payload)?;
        let session = member(&request, "sessionId")?;
        let id = member(&request, "terminalId")?;
        let terminals = self.hosted.acp.terminals(connection.id())?;
        terminals.validate_id(&id, &session)?;
        let claim = connection.claim_host(responder, method, &request)?;
        Ok(TerminalWait {
            token,
            deadline: timeout.map(|value| {
                self.task_clock_ref()
                    .deadline_after(std::time::Duration::from_millis(value as u64))
            }),
            terminals,
            session,
            id,
            _claim: claim,
        })
    }
    pub(in super::super) fn poll_acp_terminal(
        &self,
        wait: &TerminalWait,
        location: SourceLocation,
    ) -> Option<Result<Value, Fault>> {
        if let Some(token) = wait.token {
            match self.hosted.cancellations.is_cancelled(token) {
                Ok(false) => {}
                Ok(true) => return Some(Err(Fault::Cancelled)),
                Err(_) => return Some(Err(Fault::InvalidArgument("CancellationToken"))),
            }
        }
        match wait.terminals.status(&wait.id, &wait.session) {
            Err(error) => Some(Err(error)),
            Ok(Some(status)) => Some(
                raw(&sdk::WaitForTerminalExitResponse::new(status)).and_then(|raw| {
                    self.acp_host_response("WaitForTerminalExitResponse", &raw, location)
                }),
            ),
            Ok(None)
                if wait
                    .deadline
                    .is_some_and(|end| self.task_clock_ref().now_millis() >= end) =>
            {
                Some(Err(Fault::Timeout))
            }
            Ok(None) => None,
        }
    }
}

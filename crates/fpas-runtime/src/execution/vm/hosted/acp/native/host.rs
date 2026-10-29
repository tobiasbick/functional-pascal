//! Claimed SDK requests execute with native data; successful results retain resource ownership.

use super::super::host::{files, member, operation, raw};
use super::output::{Output, Terminal};
use crate::execution::vm::{
    hosted::proc::{AcpLaunch, AcpTerminals},
    worker::Worker,
};
use crate::{AcpIntrinsic as Op, Value};
use fpas_acp::{
    RawJson,
    connection::{Connection, Fault, Responder},
    protocol::v1 as sdk,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

pub(super) struct Host {
    op: Op,
    connection: Connection,
    responder: Responder,
    terminals: Arc<AcpTerminals>,
    method: &'static str,
    key: &'static str,
    request: RawJson,
    timeout: Option<Duration>,
}
impl Host {
    pub(super) fn prepare(
        worker: &Worker,
        op: Op,
        owner: u64,
        payload: &Value,
        timeout: Option<Duration>,
    ) -> Result<Self, Fault> {
        let (connection, responder) = worker.hosted.acp.responder(owner)?;
        let (method, input, key) = operation(op).ok_or(Fault::InvalidArgument("Operation"))?;
        let request = worker.acp_host_request(input, payload)?;
        if method.starts_with("fs/") {
            files::validate(method, &request)?;
        }
        let terminals = worker.hosted.acp.terminals(connection.id())?;
        Ok(Self {
            op,
            connection,
            responder,
            terminals,
            method,
            key,
            request,
            timeout,
        })
    }
    pub(super) fn run(
        self,
        cancelled: impl Fn() -> bool,
        started: Instant,
    ) -> Result<Output, Fault> {
        if cancelled() {
            return Err(Fault::Cancelled);
        }
        let maximum = self.connection.limits().message_bytes;
        let mut terminal = None;
        let response = if self.method.starts_with("fs/") {
            let _claim = self
                .connection
                .claim_host(self.responder, self.method, &self.request)?;
            files::execute_controlled(self.method, &self.request, maximum, &cancelled)?
        } else if self.op == Op::HostCreateTerminal {
            let value =
                fpas_acp::codec::decode::<sdk::CreateTerminalRequest>(self.request.clone())?.value;
            let launch = AcpLaunch {
                command: value.command,
                arguments: value.args,
                directory: value.cwd.map(|path| path.to_string_lossy().into_owned()),
                environment: value
                    .env
                    .into_iter()
                    .map(|value| (value.name, Some(value.value)))
                    .collect(),
            };
            self.terminals.validate(&launch)?;
            let (_claim, mut reserved) = self.connection.claim_host_with(
                self.responder,
                self.method,
                &self.request,
                || {
                    self.terminals
                        .reserve(&value.session_id.0, value.output_byte_limit)
                },
            )?;
            self.terminals
                .start_controlled(&mut reserved, &launch, &cancelled)?;
            terminal = Some(Terminal {
                id: Some(reserved.id.clone()),
                owner: self.terminals.clone(),
            });
            raw(&sdk::CreateTerminalResponse::new(reserved.id.clone()))?
        } else {
            let id = member(&self.request, "terminalId")?;
            let session = member(&self.request, "sessionId")?;
            self.terminals.validate_id(&id, &session, &cancelled)?;
            let _claim = self
                .connection
                .claim_host(self.responder, self.method, &self.request)?;
            match self.op {
                Op::HostTerminalOutput => {
                    raw(&self.terminals.output(&id, &session, &cancelled)?)?
                }
                Op::HostKillTerminal => {
                    self.terminals.kill(&id, &session, &cancelled)?;
                    RawJson::parse("{}")?
                }
                Op::HostReleaseTerminal => {
                    self.terminals
                        .release_controlled(&id, Some(&session), &cancelled)?;
                    RawJson::parse("{}")?
                }
                Op::HostWaitForTerminalExit => loop {
                    if cancelled() {
                        return Err(Fault::Cancelled);
                    }
                    if let Some(status) = self.terminals.status(&id, &session, &cancelled)? {
                        break raw(&sdk::WaitForTerminalExitResponse::new(status))?;
                    }
                    if self.timeout.is_some_and(|limit| started.elapsed() >= limit) {
                        return Err(Fault::Timeout);
                    }
                    std::thread::park_timeout(Duration::from_millis(1));
                },
                _ => return Err(Fault::InvalidArgument("Operation")),
            }
        };
        if response.text().len().saturating_add(128) > maximum {
            return Err(Fault::LimitExceeded("MessageBytes", maximum));
        }
        Ok(Output::Host {
            key: self.key,
            raw: response,
            terminal,
        })
    }
}

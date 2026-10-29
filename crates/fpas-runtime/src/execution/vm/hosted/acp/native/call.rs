//! Decode native-only requests before suspending; claims and OS effects run after admission.

use super::super::arguments as args;
use super::{
    host::Host,
    output::{self, Output},
};
use crate::execution::vm::{
    hosted::{HostedState, proc},
    worker::Worker,
};
use crate::{AcpIntrinsic as Op, Value};
use fpas_acp::{
    connection::{Fault, Limits, Settings},
    protocol::v1 as sdk,
    schema::Version,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

pub(super) struct Call {
    request: Request,
    pub(super) token: Option<u64>,
    pub(super) started: Instant,
}
enum Request {
    Connect {
        settings: Box<Settings>,
        limits: Limits,
        launch: Option<proc::AcpLaunch>,
    },
    Host(Host),
    Authenticate {
        launch: proc::AcpLaunch,
        method: proc::AcpAuthentication,
        timeout: Option<Duration>,
    },
}
impl Call {
    pub(super) fn prepare(worker: &Worker, op: Op, values: &[Value]) -> Result<Self, Fault> {
        let started = Instant::now();
        let mut token = None;
        let request = match (op, values) {
            (Op::ConnectMessages, [options]) => {
                let (settings, limits) = worker.acp_options(options)?;
                Request::Connect {
                    settings: Box::new(settings),
                    limits,
                    launch: None,
                }
            }
            (Op::ConnectProcess, [launch, options]) => {
                let (settings, limits) = worker.acp_options(options)?;
                Request::Connect {
                    settings: Box::new(settings),
                    limits,
                    launch: Some(args::launch(launch)?),
                }
            }
            (
                Op::HostRunTerminalAuthentication | Op::HostV2RunTerminalAuthentication,
                [launch, method, timeout, cancellation],
            ) => {
                let launch = args::launch(launch)?;
                let method = if op == Op::HostRunTerminalAuthentication {
                    fpas_acp::codec::decode::<sdk::AuthMethodTerminal>(
                        worker.acp_host_request_for(Version::V1, "AuthMethodTerminal", method)?,
                    )?
                    .value
                    .into()
                } else {
                    fpas_acp::codec::decode::<fpas_acp::protocol::v2::AuthMethodTerminal>(
                        worker.acp_host_request_for(Version::V2, "AuthMethodTerminal", method)?,
                    )?
                    .value
                    .into()
                };
                token = args::optional(cancellation)?
                    .map(args::handle)
                    .transpose()?;
                Request::Authenticate {
                    launch,
                    method,
                    timeout: args::timeout(timeout)?.map(Duration::from_millis),
                }
            }
            (
                Op::HostReadTextFile
                | Op::HostWriteTextFile
                | Op::HostCreateTerminal
                | Op::HostTerminalOutput
                | Op::HostKillTerminal
                | Op::HostReleaseTerminal,
                [owner, payload],
            ) => Request::Host(Host::prepare(
                worker,
                op,
                args::handle(owner)?,
                payload,
                None,
            )?),
            (Op::HostWaitForTerminalExit, [owner, payload, timeout, cancellation]) => {
                token = args::optional(cancellation)?
                    .map(args::handle)
                    .transpose()?;
                Request::Host(Host::prepare(
                    worker,
                    op,
                    args::handle(owner)?,
                    payload,
                    args::timeout(timeout)?.map(Duration::from_millis),
                )?)
            }
            _ => return Err(Fault::InvalidArgument("Arguments")),
        };
        if let Some(token) = token {
            worker
                .hosted
                .cancellations
                .is_cancelled(token)
                .map_err(|_| Fault::InvalidArgument("CancellationToken"))?;
        }
        Ok(Self {
            request,
            token,
            started,
        })
    }
    pub(super) fn run(
        self,
        state: &Arc<HostedState>,
        stopped: impl Fn() -> bool,
    ) -> Result<Output, Fault> {
        let cancelled = || {
            stopped()
                || self
                    .token
                    .is_some_and(|token| state.cancellations.is_cancelled(token).unwrap_or(true))
        };
        if cancelled() {
            return Err(Fault::Cancelled);
        }
        let output = match self.request {
            Request::Connect {
                settings,
                limits,
                launch,
            } => {
                let (id, endpoint) = if let Some(launch) = launch {
                    (
                        state.acp.connect_process_controlled(
                            *settings,
                            limits,
                            launch,
                            state.piped_processes.clone(),
                            cancelled,
                        )?,
                        None,
                    )
                } else {
                    let (id, endpoint) = state.acp.connect_messages(
                        *settings,
                        limits,
                        state.piped_processes.clone(),
                    )?;
                    (id, Some(endpoint))
                };
                Output::Connection(output::Connection {
                    id: Some(id),
                    endpoint,
                    state: state.clone(),
                })
            }
            Request::Host(host) => host.run(cancelled, self.started)?,
            Request::Authenticate {
                launch,
                method,
                timeout,
            } => {
                let terminal = state
                    .process_terminal
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .clone();
                Output::Boolean(proc::acp_authenticate_at(
                    &launch,
                    &method,
                    timeout,
                    terminal.as_ref(),
                    &state.processes,
                    &cancelled,
                    self.started,
                )?)
            }
        };
        if cancelled() {
            return Err(Fault::Cancelled);
        }
        Ok(output)
    }
}

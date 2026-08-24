//! Isolated hosted-runtime state shared by one VM and its callbacks.

use std::sync::Mutex;

use fpas_std::{Console, KeyInput, TextInput};

pub(super) mod console_cell_records;
pub(super) mod console_records;

mod args;
mod callbacks;
mod console;
mod console_args;
mod net;
mod test_host;

use net::NetworkConnections;

use fpas_bytecode::{Intrinsic, SourceLocation, Value};

use super::VmError;
use super::worker::Worker;

pub(super) enum HostedOutcome {
    Unhandled,
    Complete(Option<Value>),
}

impl Worker {
    pub(super) fn execute_hosted_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<HostedOutcome, VmError> {
        if let Some(value) = self.execute_args_intrinsic(intrinsic, arguments, location)? {
            return Ok(HostedOutcome::Complete(value));
        }
        if let Some(value) = self.execute_console_intrinsic(intrinsic, arguments, location)? {
            return Ok(HostedOutcome::Complete(value));
        }
        if let Some(value) = self.execute_net_intrinsic(intrinsic, arguments, location)? {
            return Ok(HostedOutcome::Complete(value));
        }
        if let Some(value) = self.execute_callback_intrinsic(intrinsic, arguments, location)? {
            return Ok(HostedOutcome::Complete(value));
        }
        if let Some(value) = self.execute_test_host_intrinsic(intrinsic, arguments, location)? {
            return Ok(HostedOutcome::Complete(value));
        }
        Ok(HostedOutcome::Unhandled)
    }
}

/// Console, input, network, and process-argument state for one VM instance.
pub(super) struct HostedState {
    pub program_args: Vec<String>,
    pub console: Mutex<Console>,
    pub text_input: Mutex<TextInput>,
    pub key_input: Mutex<KeyInput>,
    pub(in crate::vm::hosted) network_connections: NetworkConnections,
}

impl HostedState {
    pub(super) fn new(console: Console, program_args: Vec<String>) -> Self {
        Self {
            program_args,
            console: Mutex::new(console),
            text_input: Mutex::new(TextInput::new()),
            key_input: Mutex::new(KeyInput::new()),
            network_connections: NetworkConnections::new(),
        }
    }

    /// Hosted state that never reads process stdin or terminal events.
    pub(super) fn for_debug(console: Console, program_args: Vec<String>) -> Self {
        Self {
            program_args,
            console: Mutex::new(console),
            text_input: Mutex::new(TextInput::without_os_stdin()),
            key_input: Mutex::new(KeyInput::without_os_events()),
            network_connections: NetworkConnections::new(),
        }
    }
}

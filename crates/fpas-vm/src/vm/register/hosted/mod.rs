//! Isolated hosted-runtime state shared by one register VM and its callbacks.

use std::sync::Mutex;

use fpas_std::{Console, KeyInput, TextInput};

use crate::vm::shared::GraphState;

mod args;
mod callbacks;
mod console;
mod console_args;
mod graph;
mod test_host;

use fpas_bytecode::{Intrinsic, SourceLocation, Value};

use super::VmError;
use super::worker::RegisterWorker;

pub(super) enum HostedOutcome {
    Unhandled,
    Complete(Option<Value>),
}

impl RegisterWorker {
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
        if let Some(value) = self.execute_callback_intrinsic(intrinsic, arguments, location)? {
            return Ok(HostedOutcome::Complete(value));
        }
        if let Some(value) = self.execute_graph_intrinsic(intrinsic, arguments, location)? {
            return Ok(HostedOutcome::Complete(value));
        }
        if let Some(value) = self.execute_test_host_intrinsic(intrinsic, arguments, location)? {
            return Ok(HostedOutcome::Complete(value));
        }
        Ok(HostedOutcome::Unhandled)
    }
}

/// Console, input, process-argument, and graph state for one register VM instance.
pub(super) struct HostedState {
    pub program_args: Vec<String>,
    pub console: Mutex<Console>,
    pub text_input: Mutex<TextInput>,
    pub key_input: Mutex<KeyInput>,
    pub graph: Mutex<GraphState>,
}

impl HostedState {
    pub(super) fn new(console: Console, program_args: Vec<String>) -> Self {
        Self {
            program_args,
            console: Mutex::new(console),
            text_input: Mutex::new(TextInput::new()),
            key_input: Mutex::new(KeyInput::new()),
            graph: Mutex::new(GraphState::default()),
        }
    }
}

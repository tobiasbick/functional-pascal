//! Isolated hosted-runtime state shared by one VM and its callbacks.

use std::path::PathBuf;
use std::sync::Mutex;

use fpas_std::{Console, KeyInput, TextInput};

pub(super) mod console_cell_records;
pub(super) mod console_records;

mod args;
pub(super) mod callbacks;
mod console;
mod console_args;
mod http_handles;
mod net;
mod test_host;

use http_handles::HttpStateRegistry;
use net::{NetworkConnections, NetworkListeners};

use fpas_bytecode::{Intrinsic, SourceLocation, Value};

use super::worker::Worker;
use super::{VmError, diagnostics};

impl Worker {
    pub(super) fn execute_hosted_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Value>, VmError> {
        let result = match intrinsic {
            Intrinsic::Args(_) => self.execute_args_intrinsic(intrinsic, arguments, location),
            Intrinsic::Console(_) => self.execute_console_intrinsic(intrinsic, arguments, location),
            Intrinsic::Net(_) => self.execute_net_intrinsic(intrinsic, arguments, location),
            Intrinsic::Http(_) => self.execute_http_state_intrinsic(intrinsic, arguments, location),
            Intrinsic::Test(_) => self.execute_test_host_intrinsic(intrinsic, arguments, location),
            _ => {
                return Err(diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    format!("Intrinsic {intrinsic:?} is not owned by hosted VM dispatch"),
                ));
            }
        }?;
        result.ok_or_else(|| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Hosted intrinsic {intrinsic:?} was not handled by its owning module"),
            )
        })
    }
}

/// Console, input, network, and process-argument state for one VM instance.
pub(super) struct HostedState {
    pub program_args: Vec<String>,
    pub console: Mutex<Console>,
    pub text_input: Mutex<TextInput>,
    pub key_input: Mutex<KeyInput>,
    pub(in crate::vm::hosted) network_connections: NetworkConnections,
    pub(in crate::vm::hosted) network_listeners: NetworkListeners,
    pub(in crate::vm::hosted) http_states: HttpStateRegistry,
    pub(super) test_scratch_dir: Mutex<PathBuf>,
}

impl HostedState {
    pub(super) fn new(console: Console, program_args: Vec<String>) -> Self {
        Self {
            program_args,
            console: Mutex::new(console),
            text_input: Mutex::new(TextInput::new()),
            key_input: Mutex::new(KeyInput::new()),
            network_connections: NetworkConnections::new(),
            network_listeners: NetworkListeners::new(),
            http_states: HttpStateRegistry::new(),
            test_scratch_dir: Mutex::new(PathBuf::from(".temp-data")),
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
            network_listeners: NetworkListeners::new(),
            http_states: HttpStateRegistry::new(),
            test_scratch_dir: Mutex::new(PathBuf::from(".temp-data")),
        }
    }

    /// Interrupt blocking network operations during VM shutdown.
    pub(super) fn shutdown_network(&self) {
        self.network_listeners.shutdown();
        self.network_connections.shutdown();
    }
}

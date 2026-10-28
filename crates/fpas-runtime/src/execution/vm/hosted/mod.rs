//! Isolated hosted-runtime state shared by one VM and its callbacks.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::stdlib::{Console, KeyInput, TextInput};

pub(super) mod console_cell_records;
pub(super) mod console_records;

pub(in crate::execution::vm) mod acp;
mod args;
pub(super) mod callbacks;
mod console;
mod console_args;
mod console_wait;
mod crypto;
mod fs;
mod http_handles;
pub(in crate::execution::vm) mod mcp;
pub(in crate::execution::vm) mod net;
mod proc;
mod random;
pub(in crate::execution::vm) mod server;
pub(in crate::execution::vm) mod terminal_input;
mod test_host;

use http_handles::HttpStateRegistry;
use net::{NetworkConnections, NetworkListeners};

use crate::{Intrinsic, SourceLocation, Value};

use super::worker::Worker;
use super::{VmError, cancellation::CancellationRegistry, channels::ChannelRegistry, diagnostics};

impl Worker {
    pub(super) fn execute_hosted_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Value>, VmError> {
        let result = match intrinsic {
            Intrinsic::Acp(_) => self.execute_acp_intrinsic(intrinsic, arguments, location),
            Intrinsic::Mcp(_) => self.execute_mcp_intrinsic(intrinsic, arguments, location),
            Intrinsic::Server(_) => self.execute_server_intrinsic(intrinsic, arguments, location),
            Intrinsic::Args(_) => self.execute_args_intrinsic(intrinsic, arguments, location),
            Intrinsic::Console(_) => self.execute_console_intrinsic(intrinsic, arguments, location),
            Intrinsic::Net(_) => self.execute_net_intrinsic(intrinsic, arguments, location),
            Intrinsic::Fs(_) => self.execute_fs_intrinsic(intrinsic, arguments, location),
            Intrinsic::Http(_) => self.execute_http_state_intrinsic(intrinsic, arguments, location),
            Intrinsic::Random(_) => self.execute_random_intrinsic(intrinsic, arguments, location),
            Intrinsic::Crypto(_) => self.execute_crypto_intrinsic(intrinsic, arguments, location),
            Intrinsic::Proc(_) => self.execute_proc_intrinsic(intrinsic, arguments, location),
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
    pub(in crate::execution::vm) acp: acp::Registry,
    pub(in crate::execution::vm) mcp: mcp::Registry,
    pub(in crate::execution::vm) servers: server::ServerRegistry,
    pub program_args: Vec<String>,
    pub console: Arc<Mutex<Console>>,
    pub text_input: Arc<Mutex<TextInput>>,
    pub key_input: Arc<Mutex<KeyInput>>,
    pub(in crate::execution::vm::hosted) network_connections: NetworkConnections,
    pub(in crate::execution::vm::hosted) network_listeners: NetworkListeners,
    pub(in crate::execution::vm::hosted) http_states: HttpStateRegistry,
    pub(in crate::execution::vm::hosted) files: fs::Files,
    pub(in crate::execution::vm::hosted) processes: proc::Processes,
    pub(in crate::execution::vm::hosted) piped_processes: Arc<proc::PipedProcesses>,
    pub(in crate::execution::vm) process_terminal:
        Mutex<Option<crate::execution::InteractiveProcessTerminal>>,
    pub(in crate::execution::vm::hosted) random: Mutex<random::state::RandomState>,
    pub(in crate::execution::vm) cancellations: CancellationRegistry,
    pub(in crate::execution::vm) channels: ChannelRegistry,
    pub(in crate::execution::vm) cases: Arc<super::tasks::selection::CaseRegistry>,
    pub(super) test_scratch_dir: Mutex<PathBuf>,
    /// Receiver for stderr lines of `Std.Proc.Run` children; inherited when `None`.
    pub(super) program_stderr: Mutex<Option<crate::stdlib::ProgramStderr>>,
    /// Routine and line samples, installed only for `fpas run/debug --profile`.
    pub(in crate::execution::vm) profile: std::sync::OnceLock<Arc<super::profile::ProfileSink>>,
}

impl HostedState {
    pub(super) fn new(console: Console, program_args: Vec<String>) -> Self {
        Self::with_io(
            Arc::new(Mutex::new(console)),
            Arc::new(Mutex::new(TextInput::new())),
            Arc::new(Mutex::new(KeyInput::new())),
            program_args,
        )
    }

    /// Hosted state that never reads process stdin or terminal events.
    pub(super) fn for_debug(
        console: Arc<Mutex<Console>>,
        text_input: Arc<Mutex<TextInput>>,
        key_input: Arc<Mutex<KeyInput>>,
        program_args: Vec<String>,
    ) -> Self {
        Self::with_io(console, text_input, key_input, program_args)
    }

    fn with_io(
        console: Arc<Mutex<Console>>,
        text_input: Arc<Mutex<TextInput>>,
        key_input: Arc<Mutex<KeyInput>>,
        program_args: Vec<String>,
    ) -> Self {
        Self {
            acp: Default::default(),
            mcp: Default::default(),
            servers: Default::default(),
            program_args,
            console,
            text_input,
            key_input,
            network_connections: NetworkConnections::new(),
            network_listeners: NetworkListeners::new(),
            http_states: HttpStateRegistry::new(),
            files: fs::Files::new(),
            processes: proc::Processes::new(),
            piped_processes: Default::default(),
            process_terminal: Mutex::new(None),
            random: Mutex::new(random::state::RandomState::new()),
            cancellations: CancellationRegistry::new(),
            channels: ChannelRegistry::new(),
            cases: Default::default(),
            test_scratch_dir: Mutex::new(PathBuf::from(".temp-data")),
            program_stderr: Mutex::new(None),
            profile: std::sync::OnceLock::new(),
        }
    }

    /// Interrupt blocking hosted operations during VM shutdown.
    pub(super) fn shutdown_blocking_operations(&self) {
        self.mcp.shutdown();
        self.acp.shutdown();
        self.piped_processes.shutdown();
        self.processes.shutdown();
        self.files.shutdown();
        self.channels.shutdown();
        self.network_listeners.shutdown();
        self.network_connections.shutdown();
    }

    /// Close file handles after a run or callback session has finished its workers.
    pub(super) fn close_file_handles(&self) {
        self.files.shutdown();
    }

    /// Request process-group termination without waiting in an active host callback.
    pub(super) fn close_process_handles(&self) {
        self.mcp.shutdown();
        self.acp.shutdown();
        self.piped_processes.shutdown();
        self.processes.shutdown();
    }

    /// Wait for process calls after workers and host callbacks have returned.
    pub(super) fn wait_process_handles(&self) {
        self.mcp.wait_idle();
        self.acp.wait_idle();
        self.piped_processes.wait_idle();
        self.processes.wait_idle();
    }
}

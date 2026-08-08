//! Safe register interpreter lifecycle through P5 globals and aggregates.

mod access;
mod callback;
mod calls;
mod diagnostics;
mod dispatch;
mod execute;
mod frame;
mod hosted;
mod layouts;
mod worker;

#[cfg(test)]
mod tests;

use std::sync::{Arc, RwLock};

use fpas_bytecode::{FunctionId, Value, VerifiedExecutable};

use self::hosted::HostedState;
use self::layouts::RuntimeLayouts;
use self::worker::RegisterWorker;
use super::VmError;

pub use callback::RegisterCallbackSession;

/// Successful root execution result and diagnostic instruction count.
#[derive(Debug, Clone, PartialEq)]
pub struct RegisterExecution {
    /// Root or callback return value.
    pub value: Value,
    /// Number of packed instructions dispatched by this run.
    pub instruction_count: u64,
}

/// Single-use VM for a pre-verified register executable.
///
/// This API is intentionally not wired to the production CLI until later phases complete
/// intrinsics, tasks, persistent artifacts, and the production cutover.
pub struct RegisterVm {
    executable: Arc<VerifiedExecutable>,
    has_run: bool,
    globals: Arc<RwLock<Vec<Option<Value>>>>,
    layouts: Result<Arc<RuntimeLayouts>, VmError>,
    hosted: Arc<HostedState>,
}

impl RegisterVm {
    /// Construct an isolated VM from an owned verified executable.
    #[must_use]
    pub fn new(executable: VerifiedExecutable) -> Self {
        Self::from_shared_with_host(Arc::new(executable), fpas_std::Console::new(), Vec::new())
    }

    /// Construct an isolated VM sharing immutable verified executable metadata.
    #[must_use]
    pub fn from_shared(executable: Arc<VerifiedExecutable>) -> Self {
        Self::from_shared_with_host(executable, fpas_std::Console::new(), Vec::new())
    }

    /// Construct an isolated VM with process arguments visible through `Std.Args`.
    #[must_use]
    pub fn with_args(executable: VerifiedExecutable, arguments: Vec<String>) -> Self {
        Self::from_shared_with_host(Arc::new(executable), fpas_std::Console::new(), arguments)
    }

    /// Construct an isolated VM that streams hosted console output.
    #[must_use]
    pub fn with_writer(
        executable: VerifiedExecutable,
        writer: Box<dyn std::io::Write + Send>,
    ) -> Self {
        Self::from_shared_with_host(
            Arc::new(executable),
            fpas_std::Console::with_writer(writer),
            Vec::new(),
        )
    }

    /// Construct an isolated VM that streams output and exposes process arguments.
    #[must_use]
    pub fn with_writer_and_args(
        executable: VerifiedExecutable,
        writer: Box<dyn std::io::Write + Send>,
        arguments: Vec<String>,
    ) -> Self {
        Self::from_shared_with_host(
            Arc::new(executable),
            fpas_std::Console::with_writer(writer),
            arguments,
        )
    }

    fn from_shared_with_host(
        executable: Arc<VerifiedExecutable>,
        console: fpas_std::Console,
        arguments: Vec<String>,
    ) -> Self {
        let globals = Arc::new(RwLock::new(vec![
            None;
            executable.executable().globals.len()
        ]));
        let layouts = RuntimeLayouts::build(
            executable.executable(),
            fpas_bytecode::InstructionAddress::new(0),
        )
        .map(Arc::new);
        Self {
            executable,
            has_run: false,
            globals,
            layouts,
            hosted: Arc::new(HostedState::new(console, arguments)),
        }
    }

    /// Queue one line for hosted `Read` and `ReadLn` calls.
    pub fn push_readln_input(&mut self, line: &str) {
        self.hosted
            .text_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push_line(line);
    }

    /// Queue characters for hosted `Std.Console.ReadKey` calls.
    pub fn push_readkey_input(&mut self, characters: &str) {
        self.hosted
            .key_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push_chars(characters);
    }

    /// Queue one structured hosted key event.
    pub fn push_key_event(&mut self, event: fpas_std::ConsoleKeyEvent) {
        self.hosted
            .key_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push_key_event(event);
    }

    /// Queue one unified hosted console event.
    pub fn push_console_event(&mut self, event: fpas_std::ConsoleEvent) {
        self.hosted
            .key_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push_console_event(event);
    }

    /// Queue one hosted graph event, retaining it until a session opens when necessary.
    pub fn push_graph_event(&mut self, event: fpas_std::GraphEvent) {
        let mut graph = self
            .hosted
            .graph
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if graph
            .session
            .push_event(event.clone(), fpas_bytecode::SourceLocation::new(1, 1))
            .is_err()
        {
            graph.pending_test_events.push(event);
        }
    }

    /// Return the currently captured console output.
    #[must_use]
    pub fn output(&self) -> super::VmOutput {
        self.hosted
            .console
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .output()
            .clone()
    }

    /// Return a deterministic logical console-screen snapshot.
    #[must_use]
    pub fn screen_snapshot(&self) -> fpas_std::ScreenSnapshot {
        self.hosted
            .console
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .screen_snapshot()
    }

    /// Execute the verified root function once.
    ///
    /// # Errors
    ///
    /// Returns the preserved runtime diagnostic for scalar failures, or an internal invariant
    /// diagnostic if a verified executable requests an opcode assigned to a later phase.
    pub fn run(&mut self) -> Result<RegisterExecution, VmError> {
        if std::mem::replace(&mut self.has_run, true) {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                fpas_bytecode::InstructionAddress::new(0),
                fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN,
                "This register VM instance has already been run",
                "Register VM instances are single-use. Construct a new instance for each run.",
            ));
        }
        RegisterWorker::for_function_with_state(
            Arc::clone(&self.executable),
            self.executable.executable().entry,
            Vec::new(),
            Arc::clone(&self.globals),
            self.layouts.clone()?,
            Arc::clone(&self.hosted),
        )?
        .run()
    }

    /// Execute one numeric function as a synchronous hosted callback.
    ///
    /// # Errors
    ///
    /// Returns a runtime diagnostic for an invalid target, wrong arity, shutdown reuse, panic, or
    /// resource-limit failure. No function name lookup occurs.
    pub fn call(
        &mut self,
        function: FunctionId,
        arguments: Vec<Value>,
    ) -> Result<RegisterExecution, VmError> {
        if std::mem::replace(&mut self.has_run, true) {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                fpas_bytecode::InstructionAddress::new(0),
                fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN,
                "This register VM instance has already been run",
                "Register VM instances are single-use. Construct a new instance for each callback.",
            ));
        }
        RegisterWorker::for_function_with_state(
            Arc::clone(&self.executable),
            function,
            arguments,
            Arc::clone(&self.globals),
            self.layouts.clone()?,
            Arc::clone(&self.hosted),
        )?
        .run()
    }
}

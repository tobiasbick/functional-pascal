//! Register-window state and execution loop.

use std::cell::Cell;
use std::sync::{Arc, RwLock};

use fpas_bytecode::{FunctionId, InstructionAddress, SharedFunction, Value, VerifiedExecutable};
use fpas_diagnostics::codes::{
    RUNTIME_UNDEFINED_FUNCTION, RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_WRONG_CALL_ARITY,
};

use super::dispatch::DispatchStep;
use super::frame::CallFrame;
use super::hosted::HostedState;
use super::layouts::RuntimeLayouts;
use super::{RegisterExecution, VmError, diagnostics};

pub(super) struct RegisterWorker {
    pub executable: Arc<VerifiedExecutable>,
    pub function: FunctionId,
    pub ip: usize,
    pub base: usize,
    pub registers: Vec<Value>,
    pub globals: Arc<RwLock<Vec<Option<Value>>>>,
    pub layouts: Arc<RuntimeLayouts>,
    pub hosted: Arc<HostedState>,
    pub call_stack: Vec<CallFrame>,
    pub instruction_count: u64,
    callback_instruction_count: Cell<u64>,
    pub current_address: InstructionAddress,
}

impl RegisterWorker {
    #[cfg(test)]
    pub fn new(executable: Arc<VerifiedExecutable>) -> Result<Self, VmError> {
        let entry = executable.executable().entry;
        let globals = Arc::new(RwLock::new(vec![
            None;
            executable.executable().globals.len()
        ]));
        let layouts = Arc::new(RuntimeLayouts::build(
            executable.executable(),
            InstructionAddress::new(0),
        )?);
        let hosted = Arc::new(HostedState::new(fpas_std::Console::new(), Vec::new()));
        Self::for_function_with_state(executable, entry, Vec::new(), globals, layouts, hosted)
    }

    pub fn for_function_with_state(
        executable: Arc<VerifiedExecutable>,
        entry: FunctionId,
        arguments: Vec<Value>,
        globals: Arc<RwLock<Vec<Option<Value>>>>,
        layouts: Arc<RuntimeLayouts>,
        hosted: Arc<HostedState>,
    ) -> Result<Self, VmError> {
        Self::for_function_with_captures(
            executable,
            entry,
            arguments,
            Vec::new(),
            globals,
            layouts,
            hosted,
        )
    }

    fn for_function_with_captures(
        executable: Arc<VerifiedExecutable>,
        entry: FunctionId,
        arguments: Vec<Value>,
        captures: Vec<Value>,
        globals: Arc<RwLock<Vec<Option<Value>>>>,
        layouts: Arc<RuntimeLayouts>,
        hosted: Arc<HostedState>,
    ) -> Result<Self, VmError> {
        let image = executable.executable();
        let info = image
            .functions
            .get(usize::from(entry.get()))
            .ok_or_else(|| {
                diagnostics::internal(
                    image,
                    InstructionAddress::new(0),
                    "Root function metadata is missing",
                )
            })?;
        let start = info.code.start;
        if arguments.len() != usize::from(info.arity)
            || captures.len() != usize::from(info.capture_count)
        {
            return Err(diagnostics::internal(
                image,
                start,
                format!(
                    "Callback entry signature mismatch: expected {} arguments and {} captures, got {} arguments and {} captures",
                    info.arity,
                    info.capture_count,
                    arguments.len(),
                    captures.len()
                ),
            ));
        }
        let register_count = info.register_count;
        let ip = usize::try_from(start.get()).map_err(|_| {
            diagnostics::internal(
                image,
                start,
                "Root instruction address does not fit this host",
            )
        })?;
        let mut registers = vec![Value::Unit; usize::from(register_count)];
        for (index, value) in arguments.into_iter().chain(captures).enumerate() {
            registers[index] = value;
        }
        Ok(Self {
            executable,
            function: entry,
            ip,
            base: 0,
            registers,
            globals,
            layouts,
            hosted,
            call_stack: Vec::new(),
            instruction_count: 0,
            callback_instruction_count: Cell::new(0),
            current_address: start,
        })
    }

    /// Invoke a first-class function synchronously through its numeric register target.
    pub(super) fn call_callback_sync(
        &self,
        callback: &Value,
        arguments: Vec<Value>,
    ) -> Result<Value, VmError> {
        let Value::Function(function) = callback else {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                self.current_address,
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("Expected function value, got `{}`", callback.type_name()),
                "Pass a named function or function-typed variable as the callback argument.",
            ));
        };
        self.call_numeric_function(function, arguments)
    }

    fn call_numeric_function(
        &self,
        function: &SharedFunction,
        arguments: Vec<Value>,
    ) -> Result<Value, VmError> {
        let target = function.function.ok_or_else(|| {
            diagnostics::at_address(
                self.executable.executable(),
                self.current_address,
                RUNTIME_UNDEFINED_FUNCTION,
                format!(
                    "Function `{}` has no numeric register-VM target",
                    function.name
                ),
                "Compile the callback with the register compiler before invoking it.",
            )
        })?;
        let info = self
            .executable
            .executable()
            .functions
            .get(usize::from(target.get()))
            .ok_or_else(|| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Callback target is outside the function table",
                )
            })?;
        if arguments.len() != usize::from(info.arity) {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                self.current_address,
                RUNTIME_WRONG_CALL_ARITY,
                format!(
                    "Function `{}` expects {} arguments, got {}",
                    function.name,
                    info.arity,
                    arguments.len()
                ),
                "Check the callback signature and the intrinsic's callback contract.",
            ));
        }
        let execution = Self::for_function_with_captures(
            Arc::clone(&self.executable),
            target,
            arguments,
            function.captures.clone(),
            Arc::clone(&self.globals),
            Arc::clone(&self.layouts),
            Arc::clone(&self.hosted),
        )?
        .run()?;
        self.callback_instruction_count.set(
            self.callback_instruction_count
                .get()
                .saturating_add(execution.instruction_count),
        );
        Ok(execution.value)
    }

    pub fn run(mut self) -> Result<RegisterExecution, VmError> {
        loop {
            match self.dispatch_one()? {
                DispatchStep::Continue => {}
                DispatchStep::Return(value) => {
                    return Ok(RegisterExecution {
                        value,
                        instruction_count: self
                            .instruction_count
                            .saturating_add(self.callback_instruction_count.get()),
                    });
                }
            }
        }
    }

    pub fn future_phase(&self, opcode: fpas_bytecode::Opcode) -> VmError {
        diagnostics::internal(
            self.executable.executable(),
            self.current_address,
            format!(
                "Opcode {opcode:?} is verified but not executable before its assigned register-VM phase"
            ),
        )
    }
}

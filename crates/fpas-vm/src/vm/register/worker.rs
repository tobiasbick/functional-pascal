//! Register-window state and execution loop.

use std::sync::Arc;

use fpas_bytecode::{FunctionId, InstructionAddress, Value, VerifiedExecutable};

use super::dispatch::DispatchStep;
use super::frame::CallFrame;
use super::{RegisterExecution, VmError, diagnostics};

pub(super) struct RegisterWorker {
    pub executable: Arc<VerifiedExecutable>,
    pub function: FunctionId,
    pub ip: usize,
    pub base: usize,
    pub registers: Vec<Value>,
    pub call_stack: Vec<CallFrame>,
    pub instruction_count: u64,
    pub current_address: InstructionAddress,
}

impl RegisterWorker {
    pub fn new(executable: Arc<VerifiedExecutable>) -> Result<Self, VmError> {
        let entry = executable.executable().entry;
        Self::for_function(executable, entry, Vec::new())
    }

    pub fn for_function(
        executable: Arc<VerifiedExecutable>,
        entry: FunctionId,
        arguments: Vec<Value>,
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
        if arguments.len() != usize::from(info.arity) || info.capture_count != 0 {
            return Err(diagnostics::internal(
                image,
                start,
                format!(
                    "Callback entry signature mismatch: expected {} arguments and no captures, got {} arguments",
                    info.arity,
                    arguments.len()
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
        for (index, value) in arguments.into_iter().enumerate() {
            registers[index] = value;
        }
        Ok(Self {
            executable,
            function: entry,
            ip,
            base: 0,
            registers,
            call_stack: Vec::new(),
            instruction_count: 0,
            current_address: start,
        })
    }

    pub fn run(mut self) -> Result<RegisterExecution, VmError> {
        loop {
            match self.dispatch_one()? {
                DispatchStep::Continue => {}
                DispatchStep::Return(value) => {
                    return Ok(RegisterExecution {
                        value,
                        instruction_count: self.instruction_count,
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

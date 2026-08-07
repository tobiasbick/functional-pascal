//! P3 root register state and execution loop.

use std::sync::Arc;

use fpas_bytecode::{FunctionId, InstructionAddress, Value, VerifiedExecutable};

use super::dispatch::DispatchStep;
use super::{RegisterExecution, VmError, diagnostics};

pub(super) struct RegisterWorker {
    pub executable: Arc<VerifiedExecutable>,
    pub function: FunctionId,
    pub ip: usize,
    pub registers: Vec<Value>,
    pub instruction_count: u64,
    pub current_address: InstructionAddress,
}

impl RegisterWorker {
    pub fn new(executable: Arc<VerifiedExecutable>) -> Result<Self, VmError> {
        let image = executable.executable();
        let entry = image.entry;
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
        let register_count = info.register_count;
        let ip = usize::try_from(start.get()).map_err(|_| {
            diagnostics::internal(
                image,
                start,
                "Root instruction address does not fit this host",
            )
        })?;
        Ok(Self {
            executable,
            function: entry,
            ip,
            registers: vec![Value::Unit; usize::from(register_count)],
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

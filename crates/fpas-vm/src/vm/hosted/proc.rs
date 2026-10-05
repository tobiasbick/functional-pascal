//! Hosted `Std.Proc.Run`, which can route child stderr to the VM's program-stderr receiver.
//!
//! **Documentation:** `docs/pascal/std/host/proc.md`,
//! `docs/pascal/program-structure/cli.md` (machine-readable diagnostics).

use fpas_bytecode::{Intrinsic, ProcIntrinsic, SourceLocation, Value};

use super::super::worker::Worker;
use super::super::{Vm, VmError};

impl Worker {
    pub(super) fn execute_proc_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Proc(ProcIntrinsic::Run) = intrinsic else {
            return Ok(None);
        };
        let (command, args) =
            fpas_std::run_process_arguments(arguments, location).map_err(Box::new)?;
        let receiver = self
            .hosted
            .program_stderr
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        Ok(Some(Some(fpas_std::run_process(
            &command,
            &args,
            receiver.as_ref(),
        ))))
    }
}

impl Vm {
    /// Routes standard-error lines of `Std.Proc.Run` children to `receiver`.
    ///
    /// Without a receiver, children inherit the host process's stderr.
    /// Documentation: `docs/pascal/program-structure/cli.md` (machine-readable diagnostics).
    pub fn set_program_stderr(&mut self, receiver: fpas_std::ProgramStderr) {
        *self
            .hosted
            .program_stderr
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(receiver);
    }
}

//! Hosted `Std.Proc.Run`, which can route child stderr to the VM's program-stderr receiver.
//!
//! **Documentation:** `docs/pascal/std/host/proc.md`,
//! `docs/pascal/program-structure/cli.md` (machine-readable diagnostics).

use fpas_bytecode::{Intrinsic, ProcIntrinsic, SourceLocation, Value};

use super::super::VmError;
use super::super::worker::Worker;

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

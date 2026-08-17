//! Stopped-state mapping from inspection handles onto durable data locations.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use super::*;
use crate::vm::debug::location::{DebugDataLocation, DebugDataLocationIdentity, describe_target};
use crate::vm::worker::Worker;

impl DebugSession {
    /// Describe the durable identity of one current-stop variable.
    ///
    /// Inspection handles still expire on resume. The returned identity names a
    /// global slot, a live-frame register, or an unregistered capture cell.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or expired-target error without mutation.
    pub fn describe_data_location(
        &self,
        variables_reference: u64,
        name: &str,
    ) -> Result<DebugDataLocation, DebugSessionError> {
        self.require_inspectable("location.describe")?;
        let generation = (variables_reference >> 32) as u32;
        let (task_id, inspection) = self
            .inspections
            .iter()
            .find(|(_, inspection)| inspection.generation() == generation)
            .ok_or_else(|| expired_target(name))?;
        let target = inspection.resolve_mutation_target(variables_reference, name)?;
        Ok(describe_target(&target, *task_id, inspection))
    }

    /// Return whether a previously described location is still a live identity.
    ///
    /// Capture cells are never live watchpoint identities because aliases are
    /// unregistered. Globals stay live for the session. Frame registers stay
    /// live only while that activation remains on the task stack.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns an invalid-state error when the session cannot be inspected.
    pub fn data_location_is_live(
        &self,
        location: &DebugDataLocation,
    ) -> Result<bool, DebugSessionError> {
        self.require_inspectable("location.live")?;
        match location.identity {
            Some(DebugDataLocationIdentity::Global { index }) => {
                let count = self.executable.executable().globals.len();
                Ok(usize::try_from(index).is_ok_and(|index| index < count))
            }
            Some(DebugDataLocationIdentity::FrameRegister {
                task_id,
                function,
                register,
            }) => Ok(self
                .runtime
                .worker(task_id)
                .is_some_and(|worker| frame_register_is_live(worker, function, register))),
            None => Ok(false),
        }
    }
}

fn expired_target(name: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableTargetExpired,
        message: format!("debug variable target `{name}` belongs to an expired stop snapshot"),
        hint: "Request scopes and variables again for the current stop.".to_string(),
    }
}

fn frame_register_is_live(worker: &Worker, function: u64, register: u64) -> bool {
    let Ok(function) = u16::try_from(function) else {
        return false;
    };
    let Ok(register) = usize::try_from(register) else {
        return false;
    };
    let function = fpas_bytecode::FunctionId::new(function);
    std::iter::once((worker.function, worker.base))
        .chain(
            worker
                .call_stack
                .iter()
                .map(|frame| (frame.function, frame.base)),
        )
        .any(|(activation, base)| {
            activation == function && register_in_activation(worker, activation, base, register)
        })
}

fn register_in_activation(
    worker: &Worker,
    function: fpas_bytecode::FunctionId,
    base: usize,
    register: usize,
) -> bool {
    worker
        .executable
        .executable()
        .functions
        .get(usize::from(function.get()))
        .is_some_and(|info| {
            register >= base && register < base.saturating_add(usize::from(info.register_count))
        })
}

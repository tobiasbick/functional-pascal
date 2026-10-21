//! Runtime ownership wrappers; no mutex is added to the instruction loop.
use crate::vm::{VmError, worker::Worker};
use fpas_bytecode::{ExecutionAccess, TraceRoots, Value};

impl Worker {
    pub(in crate::vm) fn execution_access(&self) -> Result<ExecutionAccess, VmError> {
        let owner = self
            .execution
            .as_ref()
            .ok_or_else(|| self.invalid_local_value("Execution owner is no longer active"))?;
        if owner.domain().id() != self.domain.id() {
            return Err(self.invalid_local_value("Execution permit belongs to a different domain"));
        }
        owner.enter().map_err(|error| self.ownership_error(error))
    }
    pub(in crate::vm) fn publish_execution(&self, extra: &[Value]) -> Result<(), VmError> {
        let Some(execution) = &self.execution else {
            return Ok(());
        };
        let mut values = Vec::new();
        if let Err(error) = self.trace_roots(&mut |value| values.push(value.clone())) {
            execution.invalidate();
            fpas_bytecode::release_values(values);
            return Err(self.ownership_error(error));
        }
        values.extend(extra.iter().cloned());
        execution.publish(values);
        Ok(())
    }
    /// Owner cases close first. Debugger inspection handles remain independent roots.
    pub(in crate::vm) fn retire_execution(&mut self) {
        let _access = self.domain.access();
        self.hosted.cases.close_owner(self.task_id);
        self.task_suspension = None;
        self.pending_completion = None;
        self.supervision = None;
        self.callback_continuations.clear();
        self.callback_worker.get_mut().take();
        self.debug_local_trace = None;
        self.reset_registers(0);
        self.call_stack.clear();
        self.execution.take();
    }
}

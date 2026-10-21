//! Native re-entry and waits publish full callers before allowing maintenance.
use crate::vm::{VmError, worker::Worker};
use fpas_bytecode::{ExecutionAccess, ExecutionResume, PublishedWait, RootKind, RootValues, Value};

pub(in crate::vm) struct NativeCallerRoots {
    _resume: Option<ExecutionResume>,
    _roots: RootValues,
    _access: ExecutionAccess,
}
/// Field order reacquires execution before touching any native value or removing roots.
pub(in crate::vm) struct NativeWait {
    _park: Option<PublishedWait>,
    _resume: Option<ExecutionResume>,
    _roots: RootValues,
}
impl Worker {
    pub(in crate::vm) fn native_caller_roots(&self) -> Result<NativeCallerRoots, VmError> {
        use fpas_bytecode::TraceRoots;
        let access = self.execution_access()?;
        let mut values = Vec::new();
        self.trace_roots(&mut |value| values.push(value.clone()))
            .map_err(|error| self.ownership_error(error))?;
        let roots = self.domain.root_values(RootKind::Native, values.clone());
        if let Some(execution) = &self.execution {
            execution.publish(values);
        }
        Ok(NativeCallerRoots {
            _resume: self.execution.as_ref().map(|owner| owner.resume_marker()),
            _roots: roots,
            _access: access,
        })
    }
    /// No cell snapshots, storage guards or FPAS callbacks may cross the returned wait.
    pub(in crate::vm) fn native_wait(&self, extra: Vec<Value>) -> Result<NativeWait, VmError> {
        let roots = self.domain.root_values(RootKind::Native, extra);
        self.publish_execution(&[])?;
        let park = self.domain.publish_wait();
        Ok(NativeWait {
            _park: park,
            _resume: self.execution.as_ref().map(|owner| owner.resume_marker()),
            _roots: roots,
        })
    }
}

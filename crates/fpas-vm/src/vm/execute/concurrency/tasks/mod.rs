use super::super::super::{Vm, VmError};
use fpas_bytecode::SourceLocation;

mod scheduling;
mod spawn;
mod wait;

impl Vm {
    pub(super) fn exec_yield(&mut self) {
        if !self.tasks.is_empty() {
            self.yield_current_task();
        }
    }

    fn pop_task_id(&mut self, line: SourceLocation) -> Result<u64, VmError> {
        wait::pop_task_id(self, line)
    }
}

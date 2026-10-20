//! Native operational task outcomes; task failures remain runtime diagnostics.
use crate::vm::{VmError, worker::Worker};
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::native_errors::{self, NativeErrorFamily};

#[derive(Clone, Copy)]
#[repr(usize)]
pub(in crate::vm) enum TaskErrorKind {
    Closed = 0,
    Timeout = 1,
    Cancelled = 2,
    InvalidGroup = 4,
}

impl Worker {
    pub(in crate::vm) fn task_outcome_error(
        &self,
        kind: TaskErrorKind,
        operation: &str,
    ) -> Result<Value, VmError> {
        self.task_error_value(kind as usize, vec![Value::Str(operation.into())])
    }

    pub(in crate::vm) fn task_input_error(
        &self,
        name: &str,
        reason: &str,
    ) -> Result<Value, VmError> {
        self.task_error_value(3, vec![Value::Str(name.into()), Value::Str(reason.into())])
    }

    #[cold]
    #[inline(never)]
    fn task_error_value(&self, variant: usize, fields: Vec<Value>) -> Result<Value, VmError> {
        let location = self
            .executable
            .executable()
            .source_map
            .lookup(self.current_address)
            .map_or(SourceLocation::new(1, 1), |run| {
                SourceLocation::new_with_source(run.line, run.column, run.source.get())
            });
        Ok(Value::result_error(native_errors::value(
            self.layouts.as_ref(),
            NativeErrorFamily::Task,
            variant,
            fields,
            location,
        )?))
    }
}

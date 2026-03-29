mod bitwise_bool;
mod comparisons;
mod dynamic_ops;
mod int_ops;
mod real_ops;

use super::super::Worker;
use super::super::diagnostics::VmError;
use fpas_bytecode::{Op, SourceLocation};

impl Worker {
    pub(super) fn try_exec_numeric(
        &mut self,
        op: Op,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        if self.try_exec_int_ops(op, line)?
            || self.try_exec_real_ops(op, line)?
            || self.try_exec_comparisons(op, line)?
            || self.try_exec_bitwise_bool(op, line)?
            || self.try_exec_dynamic_ops(op, line)?
        {
            return Ok(true);
        }

        Ok(false)
    }
}

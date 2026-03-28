use super::super::super::{Vm, VmError};
use fpas_bytecode::{Intrinsic, SourceLocation};

mod array_ops;
mod sync_call;

impl Vm {
    pub(super) fn try_exec_array_callback_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::ArrayMap => {
                self.exec_array_map(line)?;
                Ok(true)
            }
            Intrinsic::ArrayFilter => {
                self.exec_array_filter(line)?;
                Ok(true)
            }
            Intrinsic::ArrayReduce => {
                self.exec_array_reduce(line)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

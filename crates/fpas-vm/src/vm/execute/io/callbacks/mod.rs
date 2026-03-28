use super::super::super::Worker;
use super::super::super::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, SourceLocation};

mod array_ops;
mod result_option_ops;
mod sync_call;

impl Worker {
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
            Intrinsic::ResultMap => {
                self.exec_result_map(line)?;
                Ok(true)
            }
            Intrinsic::ResultAndThen => {
                self.exec_result_and_then(line)?;
                Ok(true)
            }
            Intrinsic::ResultOrElse => {
                self.exec_result_or_else(line)?;
                Ok(true)
            }
            Intrinsic::OptionMap => {
                self.exec_option_map(line)?;
                Ok(true)
            }
            Intrinsic::OptionAndThen => {
                self.exec_option_and_then(line)?;
                Ok(true)
            }
            Intrinsic::OptionOrElse => {
                self.exec_option_or_else(line)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

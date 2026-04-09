use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, SourceLocation};

mod array_ops;
mod dict_ops;
mod result_option_ops;
mod sync_call;

impl Worker {
    pub(super) fn try_exec_higher_order_intrinsic(
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
            Intrinsic::ArrayFind => {
                self.exec_array_find(line)?;
                Ok(true)
            }
            Intrinsic::ArrayFindIndex => {
                self.exec_array_find_index(line)?;
                Ok(true)
            }
            Intrinsic::ArrayAny => {
                self.exec_array_any(line)?;
                Ok(true)
            }
            Intrinsic::ArrayAll => {
                self.exec_array_all(line)?;
                Ok(true)
            }
            Intrinsic::ArrayFlatMap => {
                self.exec_array_flat_map(line)?;
                Ok(true)
            }
            Intrinsic::ArrayForEach => {
                self.exec_array_for_each(line)?;
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
            Intrinsic::DictMap => {
                self.exec_dict_map(line)?;
                Ok(true)
            }
            Intrinsic::DictFilter => {
                self.exec_dict_filter(line)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

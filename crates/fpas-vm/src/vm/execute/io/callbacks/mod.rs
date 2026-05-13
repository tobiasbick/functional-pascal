use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{ArrayIntrinsic, DictIntrinsic, Intrinsic, OptionIntrinsic, ResultIntrinsic, SourceLocation};

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
            Intrinsic::Array(ArrayIntrinsic::Map) => {
                self.exec_array_map(line)?;
                Ok(true)
            }
            Intrinsic::Array(ArrayIntrinsic::Filter) => {
                self.exec_array_filter(line)?;
                Ok(true)
            }
            Intrinsic::Array(ArrayIntrinsic::Reduce) => {
                self.exec_array_reduce(line)?;
                Ok(true)
            }
            Intrinsic::Array(ArrayIntrinsic::Find) => {
                self.exec_array_find(line)?;
                Ok(true)
            }
            Intrinsic::Array(ArrayIntrinsic::FindIndex) => {
                self.exec_array_find_index(line)?;
                Ok(true)
            }
            Intrinsic::Array(ArrayIntrinsic::Any) => {
                self.exec_array_any(line)?;
                Ok(true)
            }
            Intrinsic::Array(ArrayIntrinsic::All) => {
                self.exec_array_all(line)?;
                Ok(true)
            }
            Intrinsic::Array(ArrayIntrinsic::FlatMap) => {
                self.exec_array_flat_map(line)?;
                Ok(true)
            }
            Intrinsic::Array(ArrayIntrinsic::ForEach) => {
                self.exec_array_for_each(line)?;
                Ok(true)
            }
            Intrinsic::Result(ResultIntrinsic::Map) => {
                self.exec_result_map(line)?;
                Ok(true)
            }
            Intrinsic::Result(ResultIntrinsic::AndThen) => {
                self.exec_result_and_then(line)?;
                Ok(true)
            }
            Intrinsic::Result(ResultIntrinsic::OrElse) => {
                self.exec_result_or_else(line)?;
                Ok(true)
            }
            Intrinsic::Option(OptionIntrinsic::Map) => {
                self.exec_option_map(line)?;
                Ok(true)
            }
            Intrinsic::Option(OptionIntrinsic::AndThen) => {
                self.exec_option_and_then(line)?;
                Ok(true)
            }
            Intrinsic::Option(OptionIntrinsic::OrElse) => {
                self.exec_option_or_else(line)?;
                Ok(true)
            }
            Intrinsic::Dict(DictIntrinsic::Map) => {
                self.exec_dict_map(line)?;
                Ok(true)
            }
            Intrinsic::Dict(DictIntrinsic::Filter) => {
                self.exec_dict_filter(line)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

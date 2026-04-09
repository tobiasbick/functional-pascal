use crate::vm::diagnostics::VmError;
use crate::vm::{Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

impl Worker {
    /// `Std.Dict.Map(D, F)` — transform every value; `F: function(V): V2`.
    ///
    /// **Documentation:** `docs/pascal/std/dict.md`
    pub(super) fn exec_dict_map(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let pairs = self.pop_dict_value(line, "Std.Dict.Map")?;

        let mut result = Vec::with_capacity(pairs.len());
        for (k, v) in &pairs {
            let mapped = self.call_function_sync(&func, std::slice::from_ref(v), line)?;
            result.push((k.clone(), mapped));
        }
        self.push(Value::Dict(result))?;
        Ok(())
    }

    /// `Std.Dict.Filter(D, F)` — keep entries where `F(K, V)` is true.
    ///
    /// **Documentation:** `docs/pascal/std/dict.md`
    pub(super) fn exec_dict_filter(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let pairs = self.pop_dict_value(line, "Std.Dict.Filter")?;

        let mut result = Vec::new();
        for (k, v) in &pairs {
            let keep = self.call_function_sync(&func, &[k.clone(), v.clone()], line)?;
            if self.is_truthy(&keep) {
                result.push((k.clone(), v.clone()));
            }
        }
        self.push(Value::Dict(result))?;
        Ok(())
    }

    // Helper: pop a dict value off the stack with a descriptive context name.
    fn pop_dict_value(
        &mut self,
        line: SourceLocation,
        context: &str,
    ) -> Result<Vec<(Value, Value)>, VmError> {
        match self.pop(line)? {
            Value::Dict(pairs) => Ok(pairs),
            other => Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!(
                    "{context}: first argument must be a dict, got {}",
                    other.type_name()
                ),
                "Pass a `dict of K to V` as the first argument.",
                line,
            )),
        }
    }
}

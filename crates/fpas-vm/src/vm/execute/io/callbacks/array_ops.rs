use super::super::super::super::{Vm, VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

impl Vm {
    /// `Std.Array.Map(Arr, F)` — transform each element.
    ///
    /// **Documentation:** `docs/future/closures.md`
    pub(super) fn exec_array_map(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let array = self.pop_array_value(line, "Std.Array.Map")?;

        let mut result = Vec::with_capacity(array.len());
        for elem in &array {
            let mapped = self.call_function_sync(&func, std::slice::from_ref(elem), line)?;
            result.push(mapped);
        }
        self.push(Value::Array(result))?;
        Ok(())
    }

    /// `Std.Array.Filter(Arr, F)` — keep matching elements.
    ///
    /// **Documentation:** `docs/future/closures.md`
    pub(super) fn exec_array_filter(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let array = self.pop_array_value(line, "Std.Array.Filter")?;

        let mut result = Vec::new();
        for elem in &array {
            let keep = self.call_function_sync(&func, std::slice::from_ref(elem), line)?;
            if self.is_truthy(&keep) {
                result.push(elem.clone());
            }
        }
        self.push(Value::Array(result))?;
        Ok(())
    }

    /// `Std.Array.Reduce(Arr, Init, F)` — fold into single value.
    ///
    /// **Documentation:** `docs/future/closures.md`
    pub(super) fn exec_array_reduce(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let init = self.pop(line)?;
        let array = self.pop_array_value(line, "Std.Array.Reduce")?;

        let mut acc = init;
        for elem in &array {
            acc = self.call_function_sync(&func, &[acc, elem.clone()], line)?;
        }
        self.push(acc)?;
        Ok(())
    }

    fn pop_array_value(
        &mut self,
        line: SourceLocation,
        intrinsic_name: &str,
    ) -> Result<Vec<Value>, VmError> {
        let value = self.pop(line)?;
        match value {
            Value::Array(values) => Ok(values),
            other => Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!(
                    "{intrinsic_name} expects array, got `{}`",
                    other.type_name()
                ),
                "Pass an array as the first argument.",
                line,
            )),
        }
    }
}

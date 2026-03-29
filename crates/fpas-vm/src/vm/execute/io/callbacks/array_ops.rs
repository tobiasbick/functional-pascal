use super::super::super::super::diagnostics::VmError;
use super::super::super::super::{Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

impl Worker {
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

    /// `Std.Array.Find(Arr, Pred)` — first matching element or None.
    ///
    /// **Documentation:** `docs/pascal/std/array.md`
    pub(super) fn exec_array_find(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let array = self.pop_array_value(line, "Std.Array.Find")?;

        for elem in &array {
            let result = self.call_function_sync(&func, std::slice::from_ref(elem), line)?;
            if self.is_truthy(&result) {
                self.push(Value::OptionSome(Box::new(elem.clone())))?;
                return Ok(());
            }
        }
        self.push(Value::OptionNone)?;
        Ok(())
    }

    /// `Std.Array.FindIndex(Arr, Pred)` — index of first match or -1.
    ///
    /// **Documentation:** `docs/pascal/std/array.md`
    pub(super) fn exec_array_find_index(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let array = self.pop_array_value(line, "Std.Array.FindIndex")?;

        for (i, elem) in array.iter().enumerate() {
            let result = self.call_function_sync(&func, std::slice::from_ref(elem), line)?;
            if self.is_truthy(&result) {
                self.push(Value::Integer(i as i64))?;
                return Ok(());
            }
        }
        self.push(Value::Integer(-1))?;
        Ok(())
    }

    /// `Std.Array.Any(Arr, Pred)` — true if any element matches.
    ///
    /// **Documentation:** `docs/pascal/std/array.md`
    pub(super) fn exec_array_any(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let array = self.pop_array_value(line, "Std.Array.Any")?;

        for elem in &array {
            let result = self.call_function_sync(&func, std::slice::from_ref(elem), line)?;
            if self.is_truthy(&result) {
                self.push(Value::Boolean(true))?;
                return Ok(());
            }
        }
        self.push(Value::Boolean(false))?;
        Ok(())
    }

    /// `Std.Array.All(Arr, Pred)` — true if all elements match.
    ///
    /// **Documentation:** `docs/pascal/std/array.md`
    pub(super) fn exec_array_all(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let array = self.pop_array_value(line, "Std.Array.All")?;

        for elem in &array {
            let result = self.call_function_sync(&func, std::slice::from_ref(elem), line)?;
            if !self.is_truthy(&result) {
                self.push(Value::Boolean(false))?;
                return Ok(());
            }
        }
        self.push(Value::Boolean(true))?;
        Ok(())
    }

    /// `Std.Array.FlatMap(Arr, F)` — map then flatten one level.
    ///
    /// **Documentation:** `docs/pascal/std/array.md`
    pub(super) fn exec_array_flat_map(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let array = self.pop_array_value(line, "Std.Array.FlatMap")?;

        let mut result = Vec::new();
        for elem in &array {
            let mapped = self.call_function_sync(&func, std::slice::from_ref(elem), line)?;
            match mapped {
                Value::Array(inner) => result.extend(inner),
                other => result.push(other),
            }
        }
        self.push(Value::Array(result))?;
        Ok(())
    }

    /// `Std.Array.ForEach(Arr, F)` — apply F to each element, return unit.
    ///
    /// **Documentation:** `docs/pascal/std/array.md`
    pub(super) fn exec_array_for_each(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let func = self.pop(line)?;
        let array = self.pop_array_value(line, "Std.Array.ForEach")?;

        for elem in &array {
            let _ = self.call_function_sync(&func, std::slice::from_ref(elem), line)?;
        }
        self.push(Value::Unit)?;
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

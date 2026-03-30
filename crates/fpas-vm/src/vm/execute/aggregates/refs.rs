use std::sync::{Arc, RwLock};

use super::super::super::diagnostics::VmError;
use super::super::super::{Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

impl Worker {
    pub(super) fn exec_make_ref(
        &mut self,
        type_idx: u16,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let type_name = self.const_str(type_idx, line)?;
        let value = self.pop(line)?;
        self.push(Value::Ref {
            type_name,
            value: Arc::new(RwLock::new(value)),
        })?;
        Ok(())
    }

    pub(super) fn deref_value(&self, value: &Value) -> Value {
        match value {
            Value::Ref { value, .. } => value.read().unwrap_or_else(|e| e.into_inner()).clone(),
            _ => value.clone(),
        }
    }

    pub(super) fn update_ref_target<T>(
        &self,
        value: &Value,
        update: impl FnOnce(&mut Value) -> Result<T, VmError>,
    ) -> Option<Result<T, VmError>> {
        let Value::Ref { value: inner, .. } = value else {
            return None;
        };

        let mut guard = inner.write().unwrap_or_else(|e| e.into_inner());
        Some(update(&mut guard))
    }

    pub(super) fn ref_operand_error(
        &self,
        op_name: &str,
        expected: &str,
        line: SourceLocation,
    ) -> VmError {
        runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("{op_name} requires a {expected} or ref {expected}"),
            format!("Use {op_name} with a {expected} value or a `ref {expected}` value."),
            line,
        )
    }
}

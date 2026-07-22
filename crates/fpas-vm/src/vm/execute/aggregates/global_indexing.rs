use crate::vm::diagnostics::VmError;
use crate::vm::{Worker, canonical_name, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_UNDEFINED_GLOBAL;

use super::indexing::{array_index_from_key, index_operand_error};

impl Worker {
    pub(super) fn exec_global_index_set(
        &mut self,
        name_idx: u16,
        index_count: u8,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let replacement = self.pop(line)?;
        let indices = self.drain_stack_tail(index_count as usize, line)?;
        let name = self.const_str_ref(name_idx, line)?.to_owned();
        let canonical = canonical_name(&name);
        let mut globals = self
            .shared
            .globals
            .write()
            .unwrap_or_else(|error| error.into_inner());
        let value = globals.get_mut(&canonical).ok_or_else(|| {
            runtime_error(
                RUNTIME_UNDEFINED_GLOBAL,
                format!("Undefined global variable `{name}`"),
                "Declare the global variable before assigning through an index.",
                line,
            )
        })?;
        set_array_path(value, &indices, replacement, line)?;
        drop(globals);
        self.push(Value::Unit)?;
        Ok(())
    }
}

fn set_array_path(
    value: &mut Value,
    indices: &[Value],
    replacement: Value,
    line: SourceLocation,
) -> Result<(), VmError> {
    let Some((index_key, remaining)) = indices.split_first() else {
        return Ok(());
    };
    let Value::Array(elements) = value else {
        return Err(index_operand_error("GlobalIndexSet", value, line));
    };
    let index = array_index_from_key(index_key, line)?;
    if index >= elements.len() {
        return Err(runtime_error(
            fpas_diagnostics::codes::RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
            format!("Array index {index} out of bounds (len {})", elements.len()),
            "Check index bounds before array assignment.",
            line,
        ));
    }
    if remaining.is_empty() {
        elements[index] = replacement;
        return Ok(());
    }
    set_array_path(&mut elements[index], remaining, replacement, line)
}

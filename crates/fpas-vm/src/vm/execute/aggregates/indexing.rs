use super::super::super::diagnostics::VmError;
use super::super::super::{Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS, RUNTIME_DICT_KEY_NOT_FOUND, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

impl Worker {
    pub(super) fn exec_index_get(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let key = self.pop(line)?;
        let collection = self.pop(line)?;
        match collection {
            Value::Array(elems) => {
                let idx = array_index_from_key(&key, line)?;
                if idx >= elems.len() {
                    return Err(runtime_error(
                        RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                        format!("Array index {idx} out of bounds (len {})", elems.len()),
                        "Check index bounds before array access.",
                        line,
                    ));
                }
                self.push(elems[idx].clone())?;
            }
            Value::Dict(pairs) => {
                let value = pairs
                    .iter()
                    .find(|(candidate, _)| candidate == &key)
                    .map(|(_, value)| value.clone());
                match value {
                    Some(value) => self.push(value)?,
                    None => {
                        return Err(runtime_error(
                            RUNTIME_DICT_KEY_NOT_FOUND,
                            format!("Key `{key}` not found in dict"),
                            "Use Std.Dict.ContainsKey to check before access.",
                            line,
                        ));
                    }
                }
            }
            _ => return Err(index_operand_error("IndexGet", &collection, line)),
        }
        Ok(())
    }

    pub(super) fn exec_index_set(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let value = self.pop(line)?;
        let key = self.pop(line)?;
        let collection = self.pop(line)?;
        match collection {
            Value::Array(mut elems) => {
                let idx = array_index_from_key(&key, line)?;
                if idx >= elems.len() {
                    return Err(runtime_error(
                        RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                        format!("Array index {idx} out of bounds (len {})", elems.len()),
                        "Check index bounds before array assignment.",
                        line,
                    ));
                }
                elems[idx] = value;
                self.push(Value::Array(elems))?;
            }
            Value::Dict(mut pairs) => {
                if let Some(entry) = pairs.iter_mut().find(|(candidate, _)| candidate == &key) {
                    entry.1 = value;
                } else {
                    pairs.push((key, value));
                }
                self.push(Value::Dict(pairs))?;
            }
            _ => return Err(index_operand_error("IndexSet", &collection, line)),
        }
        Ok(())
    }
}

fn array_index_from_key(key: &Value, line: SourceLocation) -> Result<usize, VmError> {
    match key {
        Value::Integer(n) => Ok(*n as usize),
        _ => Err(runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            "Array index must be an integer",
            "Use an integer expression for array indexing.",
            line,
        )),
    }
}

fn index_operand_error(op_name: &str, collection: &Value, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        format!(
            "{op_name} requires an array or dict, got {}",
            collection.type_name()
        ),
        "Use indexing only on array or dict values.",
        line,
    )
}

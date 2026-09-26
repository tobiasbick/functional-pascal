//! Stopped-state array and string structure mutations.

mod array;
mod string;

use fpas_bytecode::Value;

use super::*;

impl DebugSession {
    fn sequence_index(value: &Value) -> Result<i64, DebugSessionError> {
        match value {
            Value::Integer(index) => Ok(*index),
            _ => Err(DebugSessionError {
                kind: DebugErrorKind::VariableValueType,
                message: "debug sequence index expression must produce an Integer".to_string(),
                hint: "Use a zero-based Integer expression for the sequence index.".to_string(),
            }),
        }
    }
}

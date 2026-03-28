//! Runtime implementations for `Std.Dict.*` intrinsics.
//!
//! **Documentation:** `docs/future/advanced-types.md`

use crate::error::StdError;
use crate::helpers::pop_value;
use fpas_bytecode::{Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

fn pop_dict(val: Value, location: SourceLocation) -> Result<Vec<(Value, Value)>, StdError> {
    match val {
        Value::Dict(pairs) => Ok(pairs),
        other => Err(crate::error::std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("expected dict, got {}", other.type_name()),
            "Pass a `dict of K to V` value.",
            location,
        )),
    }
}

pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::DictLength => {
            let pairs = pop_dict(pop_value(stack, location)?, location)?;
            stack.push(Value::Integer(pairs.len() as i64));
        }
        Intrinsic::DictContainsKey => {
            let key = pop_value(stack, location)?;
            let pairs = pop_dict(pop_value(stack, location)?, location)?;
            let found = pairs.iter().any(|(k, _)| k == &key);
            stack.push(Value::Boolean(found));
        }
        Intrinsic::DictKeys => {
            let pairs = pop_dict(pop_value(stack, location)?, location)?;
            let keys: Vec<Value> = pairs.into_iter().map(|(k, _)| k).collect();
            stack.push(Value::Array(keys));
        }
        Intrinsic::DictValues => {
            let pairs = pop_dict(pop_value(stack, location)?, location)?;
            let values: Vec<Value> = pairs.into_iter().map(|(_, v)| v).collect();
            stack.push(Value::Array(values));
        }
        Intrinsic::DictRemove => {
            let key = pop_value(stack, location)?;
            let mut pairs = pop_dict(pop_value(stack, location)?, location)?;
            pairs.retain(|(k, _)| k != &key);
            stack.push(Value::Dict(pairs));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

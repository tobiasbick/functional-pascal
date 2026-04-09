//! Runtime implementations for `Std.Dict.*` intrinsics.
//!
//! **Documentation:** `docs/future/advanced-types.md`

use crate::error::{StdError, std_runtime_error};
use crate::helpers::pop_value;
use fpas_bytecode::{Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

fn pop_dict(val: Value, location: SourceLocation) -> Result<Vec<(Value, Value)>, StdError> {
    match val {
        Value::Dict(pairs) => Ok(pairs),
        other => Err(std_runtime_error(
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
        Intrinsic::DictGet => {
            let key = pop_value(stack, location)?;
            let pairs = pop_dict(pop_value(stack, location)?, location)?;
            let found = pairs.into_iter().find(|(k, _)| k == &key);
            match found {
                Some((_, v)) => stack.push(Value::OptionSome(Box::new(v))),
                None => stack.push(Value::OptionNone),
            }
        }
        Intrinsic::DictMerge => {
            let other = pop_dict(pop_value(stack, location)?, location)?;
            let mut base = pop_dict(pop_value(stack, location)?, location)?;
            for (k, v) in other {
                if let Some(entry) = base.iter_mut().find(|(ek, _)| ek == &k) {
                    entry.1 = v;
                } else {
                    base.push((k, v));
                }
            }
            stack.push(Value::Dict(base));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

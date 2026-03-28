//! `Std.Array.*` intrinsic implementations (non-mutating; `Push`/`Pop` use dedicated VM opcodes).
//!
//! **Documentation:** `docs/pascal/std/array.md` (from the repository root).
//! **Maintenance:** Keep that Markdown file aligned with this file, `intrinsics.rs`,
//! `fpas-vm` (`ArrayPushLocal` / `ArrayPopLocal`), `fpas-compiler`, and `fpas-sema` `std_registry.rs`.

use crate::error::{StdError, std_runtime_error};
use crate::helpers::{pop_array, pop_int, pop_value};
use fpas_bytecode::{Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS, RUNTIME_INTRINSIC_STACK_STATE_ERROR,
};

fn value_to_sort_key(v: &Value) -> Result<String, String> {
    Ok(match v {
        Value::Integer(n) => format!("i:{n:020}"),
        Value::Real(x) => format!("r:{x:.15e}"),
        Value::Str(s) => format!("s:{s}"),
        Value::Char(c) => format!("c:{c}"),
        Value::Boolean(b) => format!("b:{b}"),
        _ => return Err(format!("cannot sort arrays of {}", v.type_name())),
    })
}

pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::ArrayLength => {
            let arr = pop_array(pop_value(stack, location)?, location)?;
            stack.push(Value::Integer(arr.len() as i64));
        }
        Intrinsic::ArraySort => {
            let arr = pop_array(pop_value(stack, location)?, location)?;
            if arr.is_empty() {
                stack.push(Value::Array(arr));
                return Ok(Some(()));
            }
            let mut keys: Vec<String> = Vec::with_capacity(arr.len());
            for e in &arr {
                keys.push(value_to_sort_key(e).map_err(|m| {
                    std_runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        m,
                        "Use arrays of comparable primitive values (integer, real, string, char, boolean) with Std.Array.Sort.",
                        location,
                    )
                })?);
            }
            let mut idx: Vec<usize> = (0..arr.len()).collect();
            idx.sort_by(|&i, &j| keys[i].cmp(&keys[j]));
            let sorted: Vec<Value> = idx.into_iter().map(|i| arr[i].clone()).collect();
            stack.push(Value::Array(sorted));
        }
        Intrinsic::ArrayReverse => {
            let mut arr = pop_array(pop_value(stack, location)?, location)?;
            arr.reverse();
            stack.push(Value::Array(arr));
        }
        Intrinsic::ArrayContains => {
            let needle = pop_value(stack, location)?;
            let arr = pop_array(pop_value(stack, location)?, location)?;
            let found = arr.iter().any(|e| e == &needle);
            stack.push(Value::Boolean(found));
        }
        Intrinsic::ArrayIndexOf => {
            let needle = pop_value(stack, location)?;
            let arr = pop_array(pop_value(stack, location)?, location)?;
            let idx = arr
                .iter()
                .position(|e| e == &needle)
                .map(|i| i as i64)
                .unwrap_or(-1);
            stack.push(Value::Integer(idx));
        }
        Intrinsic::ArraySlice => {
            let len = pop_int(pop_value(stack, location)?, location)?;
            let start = pop_int(pop_value(stack, location)?, location)?;
            let arr = pop_array(pop_value(stack, location)?, location)?;
            let n = arr.len() as i64;
            if start < 0 || len < 0 || start > n || start + len > n {
                return Err(std_runtime_error(
                    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                    format!("Slice out of range (len={n}, start={start}, len_param={len})"),
                    "Ensure `start` and `len` select a valid range inside the array bounds.",
                    location,
                ));
            }
            let out: Vec<Value> = arr[start as usize..(start + len) as usize].to_vec();
            stack.push(Value::Array(out));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

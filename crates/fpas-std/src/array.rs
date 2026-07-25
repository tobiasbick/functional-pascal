//! `Std.Array.*` intrinsic implementations (non-mutating; `Push`/`Pop` use dedicated VM opcodes).
//!
//! **Documentation:** `docs/pascal/std/collections/array/README.md` (from the repository root).
//! **Maintenance:** Keep that Markdown file aligned with this file, `intrinsics.rs`,
//! `fpas-vm` (`ArrayPushLocal` / `ArrayPopLocal`), `fpas-compiler`, and `fpas-sema` `std_registry.rs`.

use crate::error::{StdError, std_runtime_error};
use crate::intrinsic_args::{expect_array, pop_array, pop_int, pop_value};
use crate::limits::checked_collection_len;
use fpas_bytecode::{ArrayIntrinsic, Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

fn value_to_sort_key(v: &Value) -> Result<String, String> {
    Ok(match v {
        // Offset by i64::MAX so that negative values sort correctly in
        // lexicographic order:  -3 → 9223372036854775804, -1 → …806, 0 → …807.
        Value::Integer(n) => format!("i:{:020}", (*n as u128).wrapping_add(i64::MAX as u128 + 1)),
        Value::Real(x) => {
            let bits = x.to_bits();
            // IEEE 754 total-order trick: flip all bits for negatives,
            // flip only sign bit for positives → monotonic u64 order.
            let sortable = if bits >> 63 == 1 {
                !bits
            } else {
                bits ^ (1 << 63)
            };
            format!("r:{sortable:020}")
        }
        Value::Str(s) => format!("s:{s}"),
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
        Intrinsic::Array(ArrayIntrinsic::Length) => {
            let arr = expect_array(pop_value(stack, location)?, location)?;
            stack.push(Value::Integer(arr.len() as i64));
        }
        Intrinsic::Array(ArrayIntrinsic::Sort) => {
            let arr = pop_array(pop_value(stack, location)?, location)?;
            if arr.is_empty() {
                stack.push(Value::Array(arr.into()));
                return Ok(Some(()));
            }
            let mut keys: Vec<String> = Vec::with_capacity(arr.len());
            for e in &arr {
                keys.push(value_to_sort_key(e).map_err(|m| {
                    std_runtime_error(
                        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                        m,
                        "Use arrays of comparable primitive values (integer, real, string, boolean) with Std.Array.Sort.",
                        location,
                    )
                })?);
            }
            let mut idx: Vec<usize> = (0..arr.len()).collect();
            idx.sort_by(|&i, &j| keys[i].cmp(&keys[j]));
            let sorted: Vec<Value> = idx.into_iter().map(|i| arr[i].clone()).collect();
            stack.push(Value::Array(sorted.into()));
        }
        Intrinsic::Array(ArrayIntrinsic::Reverse) => {
            let mut arr = pop_array(pop_value(stack, location)?, location)?;
            arr.reverse();
            stack.push(Value::Array(arr.into()));
        }
        Intrinsic::Array(ArrayIntrinsic::Contains) => {
            let needle = pop_value(stack, location)?;
            let arr = expect_array(pop_value(stack, location)?, location)?;
            let found = arr.iter().any(|e| e == &needle);
            stack.push(Value::Boolean(found));
        }
        Intrinsic::Array(ArrayIntrinsic::IndexOf) => {
            let needle = pop_value(stack, location)?;
            let arr = expect_array(pop_value(stack, location)?, location)?;
            let idx = arr
                .iter()
                .position(|e| e == &needle)
                .map(|i| i as i64)
                .unwrap_or(-1);
            stack.push(Value::Integer(idx));
        }
        Intrinsic::Array(ArrayIntrinsic::Slice) => {
            let len = pop_int(pop_value(stack, location)?, location)?;
            let start = pop_int(pop_value(stack, location)?, location)?;
            let arr = expect_array(pop_value(stack, location)?, location)?;
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
            stack.push(Value::Array(out.into()));
        }
        Intrinsic::Array(ArrayIntrinsic::Concat) => {
            let b = pop_array(pop_value(stack, location)?, location)?;
            let mut a = pop_array(pop_value(stack, location)?, location)?;
            a.extend(b);
            stack.push(Value::Array(a.into()));
        }
        Intrinsic::Array(ArrayIntrinsic::Fill) => {
            let count = pop_int(pop_value(stack, location)?, location)?;
            let value = pop_value(stack, location)?;
            let len = checked_collection_len(count, location, "Std.Array.Fill")?;
            let arr: Vec<Value> = vec![value; len];
            stack.push(Value::Array(arr.into()));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::limits::MAX_COLLECTION_LEN;

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    fn run_array(intrinsic: ArrayIntrinsic, stack: &mut Vec<Value>) -> Result<(), StdError> {
        run(Intrinsic::Array(intrinsic), stack, loc()).map(|_| ())
    }

    #[test]
    fn fill_creates_requested_length() {
        let mut stack = vec![Value::Integer(7), Value::Integer(3)];
        run_array(ArrayIntrinsic::Fill, &mut stack).unwrap();
        assert_eq!(
            stack,
            vec![Value::Array(
                vec![Value::Integer(7), Value::Integer(7), Value::Integer(7),].into()
            )]
        );
    }

    #[test]
    fn fill_rejects_count_above_limit() {
        let mut stack = vec![Value::Integer(1), Value::Integer(MAX_COLLECTION_LEN + 1)];
        let err = run_array(ArrayIntrinsic::Fill, &mut stack).unwrap_err();
        assert_eq!(err.code, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS);
    }

    #[test]
    fn slice_rejects_out_of_range() {
        let mut stack = vec![
            Value::Array(vec![Value::Integer(1), Value::Integer(2)].into()),
            Value::Integer(0),
            Value::Integer(3),
        ];
        let err = run_array(ArrayIntrinsic::Slice, &mut stack).unwrap_err();
        assert_eq!(err.code, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS);
    }
}

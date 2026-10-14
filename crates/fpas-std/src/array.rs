//! Non-mutating array intrinsic implementations reused by InPlace lowering.
//! Stack operations are composed by the compiler and use dedicated VM opcodes.
//!
//! **Documentation:** `docs/pascal/language/types/array/README.md` (from the repository root).
//! **Maintenance:** Keep that Markdown file aligned with this file, `intrinsics.rs`,
//! `fpas-vm` (`ArrayPush` / `ArrayPop`), `fpas-compiler`, and `fpas-sema` `std_registry.rs`.

use crate::error::{StdError, std_runtime_error};
use crate::intrinsic_args::{IntrinsicCall, expect_array, pop_array, pop_int, pop_value};
use crate::limits::checked_collection_len;
use fpas_bytecode::{ArrayIntrinsic, Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS, RUNTIME_NUMERIC_DOMAIN_ERROR,
    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Array(ArrayIntrinsic::Length) => {
            let arr = expect_array(pop_value(call, location)?, location)?;
            call.push(Value::Integer(arr.len() as i64));
        }
        Intrinsic::Array(ArrayIntrinsic::Sort) => {
            let mut arr = pop_array(pop_value(call, location)?, location)?;
            if let Some(first) = arr.first() {
                for value in &arr {
                    if let Value::Real(real) = value {
                        fpas_numeric::real::finite(*real).map_err(|error| {
                            std_runtime_error(
                                RUNTIME_NUMERIC_DOMAIN_ERROR,
                                error.message(),
                                error.hint(),
                                location,
                            )
                        })?;
                    }
                    if value.scalar_ordering(first).is_none() {
                        return Err(std_runtime_error(
                            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                            format!("cannot sort arrays of {}", value.type_name()),
                            "Use a homogeneous array of Comparable elements with Items.Sort().",
                            location,
                        ));
                    }
                }
            }
            // Stable sort preserves input order for equal values, including signed zeros.
            arr.sort_by(|left, right| {
                left.scalar_ordering(right)
                    .expect("validated scalar elements")
            });
            call.push(Value::Array(arr.into()));
        }
        Intrinsic::Array(ArrayIntrinsic::Reverse) => {
            let mut arr = pop_array(pop_value(call, location)?, location)?;
            arr.reverse();
            call.push(Value::Array(arr.into()));
        }
        Intrinsic::Array(ArrayIntrinsic::IndexOf) => {
            let needle = pop_value(call, location)?;
            let arr = expect_array(pop_value(call, location)?, location)?;
            let idx = arr
                .iter()
                .position(|e| e == needle)
                .map_or(Value::OptionNone, |i| {
                    Value::option_some(Value::Integer(i as i64))
                });
            call.push(idx);
        }
        Intrinsic::Array(ArrayIntrinsic::Slice) => {
            let len = pop_int(pop_value(call, location)?, location)?;
            let start = pop_int(pop_value(call, location)?, location)?;
            let arr = expect_array(pop_value(call, location)?, location)?;
            let n = arr.len() as i64;
            if start < 0 || len < 0 || start > n || len > n - start {
                return Err(std_runtime_error(
                    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
                    format!("Slice out of range (len={n}, start={start}, len_param={len})"),
                    "Ensure `start` and `len` select a valid range inside the array bounds.",
                    location,
                ));
            }
            let end = start + len;
            let out: Vec<Value> = arr[start as usize..end as usize].to_vec();
            call.push(Value::Array(out.into()));
        }
        Intrinsic::Array(ArrayIntrinsic::Concat) => {
            let b = pop_array(pop_value(call, location)?, location)?;
            let mut a = pop_array(pop_value(call, location)?, location)?;
            a.extend(b);
            call.push(Value::Array(a.into()));
        }
        Intrinsic::Array(ArrayIntrinsic::Fill) => {
            let count = pop_int(pop_value(call, location)?, location)?;
            let value = pop_value(call, location)?;
            let len = checked_collection_len(count, location, "Std.Arrays.Fill")?;
            let arr: Vec<Value> = vec![value.clone(); len];
            call.push(Value::Array(arr.into()));
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
        crate::execute_test_intrinsic(Intrinsic::Array(intrinsic), stack, loc()).map(|_| ())
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
        let err = run_array(ArrayIntrinsic::Fill, &mut stack)
            .expect_err("Fill must reject a count above the collection limit");
        assert_eq!(err.code, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS);
    }

    #[test]
    fn sort_rejects_nonfinite_runtime_elements_even_without_comparison_steps() {
        for real in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut stack = vec![Value::Array(vec![Value::Real(real)].into())];
            let error = run_array(ArrayIntrinsic::Sort, &mut stack)
                .expect_err("sorting retains finite-real validation for singleton arrays");
            assert_eq!(error.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
        }
    }

    #[test]
    fn slice_rejects_out_of_range() {
        let mut stack = vec![
            Value::Array(vec![Value::Integer(1), Value::Integer(2)].into()),
            Value::Integer(0),
            Value::Integer(3),
        ];
        let err = run_array(ArrayIntrinsic::Slice, &mut stack)
            .expect_err("Slice must reject an out-of-bounds interval");
        assert_eq!(err.code, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS);
    }

    #[test]
    fn slice_rejects_overflowing_range() {
        let mut stack = vec![
            Value::Array(vec![Value::Integer(1), Value::Integer(2)].into()),
            Value::Integer(1),
            Value::Integer(i64::MAX),
        ];
        let err = run_array(ArrayIntrinsic::Slice, &mut stack)
            .expect_err("Slice must reject an overflowing end index");
        assert_eq!(err.code, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS);
    }
}

//! Signed patterns, shift boundaries, and borrowed intrinsic argument contracts.

use fpas_bytecode::{BitsIntrinsic as Bits, Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_NUMERIC_DOMAIN_ERROR,
    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

fn run(operation: Bits, arguments: &[Value]) -> Result<Option<Value>, crate::StdError> {
    crate::run_intrinsic_borrowed(
        Intrinsic::Bits(operation),
        arguments,
        SourceLocation::new(7, 9),
        &crate::intrinsics::TEST_AGGREGATES,
    )
}

#[test]
fn bit_patterns_keep_all_64_bits_and_signed_results() {
    for (operation, left, right, expected) in [
        (Bits::BitAnd, 12, 10, 8),
        (Bits::BitOr, 12, 10, 14),
        (Bits::BitXor, 12, 10, 6),
        (Bits::BitAnd, i64::MIN, i64::MAX, 0),
        (Bits::BitOr, i64::MIN, i64::MAX, -1),
        (Bits::BitXor, i64::MIN, -1, i64::MAX),
        (Bits::BitAnd, -8, 3, 0),
        (Bits::BitOr, -8, 3, -5),
        (Bits::BitXor, -8, 3, -5),
    ] {
        assert_eq!(
            run(operation, &[Value::Integer(left), Value::Integer(right)]).unwrap(),
            Some(Value::Integer(expected)),
            "{operation:?}({left}, {right})"
        );
    }
    for (value, expected) in [(0, -1), (-1, 0), (i64::MIN, i64::MAX), (12, -13)] {
        assert_eq!(
            run(Bits::BitNot, &[Value::Integer(value)]).unwrap(),
            Some(Value::Integer(expected))
        );
    }
}

#[test]
fn every_valid_shift_count_preserves_the_signed_bit_pattern() {
    for value in [0, 1, -1, -8, i64::MIN, i64::MAX, 0x5555_5555_5555_5555] {
        for count in 0..=63 {
            let left = (i128::from(value) * (1_i128 << count)) as i64;
            let right = (i128::from(value) >> count) as i64;
            for (operation, expected) in [(Bits::ShiftLeft, left), (Bits::ShiftRight, right)] {
                assert_eq!(
                    run(operation, &[Value::Integer(value), Value::Integer(count)]).unwrap(),
                    Some(Value::Integer(expected)),
                    "{operation:?}({value}, {count})"
                );
            }
        }
    }
    assert_eq!(super::shift_left(1, 63), Some(i64::MIN));
    assert_eq!(super::shift_left(i64::MIN, 1), Some(0));
    assert_eq!(super::shift_right(-8, 1), Some(-4));
    assert_eq!(super::shift_right(-1, 63), Some(-1));
}

#[test]
fn invalid_shift_counts_are_domain_errors_without_masking() {
    for operation in [Bits::ShiftLeft, Bits::ShiftRight] {
        for count in [i64::MIN, -65, -1, 64, 65, 128, i64::MAX] {
            let error = run(operation, &[Value::Integer(1), Value::Integer(count)]).unwrap_err();
            assert_eq!(error.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
            assert!(error.message.contains("0..63"), "{error:?}");
            let span = error
                .span
                .expect("runtime error retains the source location");
            assert_eq!((span.line(), span.column()), (7, 9));
        }
    }
}

#[test]
fn every_intrinsic_rejects_wrong_arity_and_non_integer_arguments() {
    for operation in [
        Bits::BitAnd,
        Bits::BitOr,
        Bits::BitXor,
        Bits::BitNot,
        Bits::ShiftLeft,
        Bits::ShiftRight,
    ] {
        let arity = if operation == Bits::BitNot { 1 } else { 2 };
        for count in [0, arity - 1, arity + 1] {
            let error = run(operation, &vec![Value::Integer(0); count]).unwrap_err();
            assert_eq!(error.code, RUNTIME_INTRINSIC_STACK_STATE_ERROR);
        }
        for index in 0..arity {
            for value in [
                Value::Real(1.0),
                Value::Boolean(true),
                Value::Str("1".into()),
            ] {
                let mut arguments = vec![Value::Integer(0); arity];
                arguments[index] = value;
                let error = run(operation, &arguments).unwrap_err();
                assert_eq!(error.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
            }
        }
    }
}
